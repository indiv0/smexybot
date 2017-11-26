// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides functionality for the `tag` command.

use chrono::{DateTime, Utc};
use serenity::client::rest;
use serenity::framework::standard::CommandError;
use serenity::model::{GuildId, Message, UserId};
use serenity::utils::builder::CreateEmbed;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Mutex;
use store::{JsonFileStore, Store};
use util::{check_msg, lock_mutex, merge, timestamp_to_string};

type Key = String;
type Value = HashMap<String, Tag>;
type TagStore = JsonFileStore<Key, Value>;

lazy_static! {
    static ref TAGS: Mutex<Tags<Key, Value, TagStore>> = Mutex::new(Tags {
        store: JsonFileStore::new("tags.json".to_owned()),
        _data: PhantomData,
    });
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Tag {
    name: String,
    content: String,
    owner_id: u64,
    uses: u32,
    location: Option<String>,
    created_at: DateTime<Utc>,
}

impl Tag {
    fn new(name: String, content: String, owner_id: u64) -> Self {
        Tag {
            name: name,
            content: content,
            owner_id: owner_id,
            uses: 0,
            location: None,
            created_at: Utc::now(),
        }
    }

    fn as_embed(&self, embed: CreateEmbed) -> CreateEmbed {
        embed.title(&self.name)
            .field(|f| f.name("Owner").value(&format!("<@!{}>", self.owner_id)))
            .field(|f| f.name("Uses").value(&self.uses.to_string()))
            .author(|a| {
                let owner_id = UserId(self.owner_id);
                let (name, avatar_url) = match owner_id.find() {
                    Some(user) => {
                        let user = user.read().expect("Failed to read RwLock");
                        (user.name.clone(), user.avatar_url())
                    },
                    None => {
                        match rest::get_user(owner_id.0) {
                            Ok(user) => (user.name.clone(), user.avatar_url()),
                            Err(_) => return a,
                        }
                    },
                };
                let mut a = a.name(&name);
                if let Some(avatar_url) = avatar_url {
                    a = a.icon_url(&avatar_url);
                }
                a
            })
            .timestamp(timestamp_to_string(&self.created_at))
            .footer(|f| {
                f.text(if self.is_generic() {
                    "Generic"
                } else {
                    "Server-specific"
                })
            })
    }

    fn is_generic(&self) -> bool {
        self.location.is_none()
    }
}

#[derive(Debug)]
struct Tags<K, V, S>
    where S: Store<K, V>,
{
    store: S,
    _data: PhantomData<(K, V)>,
}

impl Tags<Key, Value, TagStore> {
    fn get_possible_tags(&self, guild: Option<GuildId>) -> HashMap<String, Tag> {
        let generic = self.store
            .get(&"generic".to_owned())
            .cloned()
            .unwrap_or_else(HashMap::new);

        match guild {
            None => generic,
            Some(guild) => {
                merge(generic,
                      self.store
                          .get(&guild.to_string())
                          .cloned()
                          .unwrap_or_else(HashMap::new))
            },
        }
    }

    fn create_tag(&mut self, guild: Option<GuildId>, name: String, tag: Tag)
        -> Result<(), String> {
        let location = get_database_location(guild);
        let mut database = self.store
            .remove(&location)
            .unwrap_or_else(HashMap::new);
        if database.contains_key(&name) {
            return Err("Tag already exists.".to_owned());
        }

        database.insert(name, tag);
        self.store.insert(location, database);

        Ok(())
    }

    fn get_tag(&self, guild: Option<GuildId>, name: &str) -> Result<Tag, String> {
        self.get_possible_tags(guild)
            .get(name)
            .cloned()
            .ok_or_else(|| "Tag not found".to_owned())
    }

    fn put_tag(&mut self, guild: Option<GuildId>, name: String, tag: Tag) {
        let location = get_database_location(guild);
        let mut database = self.store
            .remove(&location)
            .unwrap_or_else(HashMap::new);
        database.insert(name, tag);
        self.store.insert(location, database);
    }

    fn delete_tag(&mut self, guild: Option<GuildId>, name: &str) {
        let location = get_database_location(guild);
        let mut database = self.store
            .remove(&location)
            .unwrap_or_else(HashMap::new);
        database.remove(name);
        self.store.insert(location, database);
    }
}

command!(tag(ctx, msg, args) {
    let first = args.single::<String>().ok();

    let f = match first.as_ref().map(|v| &v[..]) {
        Some("create") => create,
        Some("info") => info,
        Some("list") => list,
        Some("edit") => edit,
        Some("delete") => delete,
        Some(name) => {
            return {
                let guild_id = msg.guild_id();

                let lookup = name.to_lowercase();
                let mut tags = lock_mutex(&*TAGS)?;
                match tags.get_tag(guild_id, &lookup) {
                    Ok(tag) => {
                        let mut tag = tag.clone();
                        let content = tag.content.clone();
                        tag.uses += 1;
                        tags.put_tag(guild_id, lookup, tag);
                        check_msg(msg.channel_id.say(&content));

                        Ok(())
                    },
                    Err(err) => Err(CommandError(err)),
                }
            };
        },
        None => return Err(CommandError("Please specified either a subcommand or a tag name".to_owned())),
    };

    // This is necessary because the `command!` macro returns `Ok(())`. Without
    // this match and fall-through, rustc would complain about unreachable code.
    match f(ctx, msg, args.clone()) {
        Ok(()) => {},
        v => return v,
    }
});

command!(create(_ctx, msg, args) {
    let name = args.single::<String>()
        .expect("Failed to read command argument");

    let content = if args.is_empty() {
        return Err(CommandError("Please specify some content for the tag.".to_owned()));
    } else {
        args.join(" ")
    };

    let name = name.trim().to_lowercase().to_owned();
    verify_tag_name(&name)?;

    let guild = msg.guild_id();
    let location = get_database_location(guild);
    let tag = {
        let mut tag = Tag::new(
            name.clone(),
            content,
            msg.author.id.0,
        );
        tag.location = Some(location);
        tag
    };
    lock_mutex(&*TAGS)?.create_tag(guild, name.clone(), tag)?;

    check_msg(msg.channel_id.say(&format!("Tag \"{}\" successfully created.", name)));
});

command!(info(_ctx, msg, args) {
    let name = args.single::<String>()
        .expect("Failed to read command argument");

    let name = name.trim().to_lowercase().to_owned();
    let guild_id = msg.guild_id();
    let tag = lock_mutex(&*TAGS)?.get_tag(guild_id, &name)?;

    check_msg(msg.channel_id.send_message(|m| m.embed(|e| tag.as_embed(e))));
});

command!(list(_ctx, msg, _args) {
    let guild_id = msg.guild_id();
    let mut tags = {
        let mut tags = lock_mutex(&*TAGS)?.get_possible_tags(guild_id);
        let mut tags = tags.drain()
            .map(|(k, _)| k)
            .collect::<Vec<String>>();
        tags.sort();
        tags
    };

    let response = if tags.is_empty() {
        "No tags available.".to_owned()
    } else {
        format!("Available tags: {}", tags.join(", "))
    };
    check_msg(msg.channel_id.say(&response));
});

command!(edit(_ctx, msg, args) {
    let name = args.single::<String>()
        .expect("Failed to parse command argument").trim().to_lowercase();

    let guild_id = msg.guild_id();
    let mut tags = lock_mutex(&*TAGS)?;
    let mut tag = tags.get_tag(guild_id, &name)?;

    if !owner_check(msg, &tag) {
        return Err(CommandError("You do not have permission to do that.".to_owned()));
    }

    let content = if args.is_empty() {
        return Err(CommandError("Please specify some content for the tag.".to_owned()));
    } else {
        args.join(" ")
    };

    tag.content = content;
    tags.put_tag(guild_id, name.clone(), tag);

    check_msg(msg.channel_id.say(&format!("Tag \"{}\" successfully updated.", name)));
});

command!(delete(_ctx, msg, args) {
    let name = args.single::<String>()
        .expect("Failed to parse command argument").trim().to_lowercase();

    let guild_id = msg.guild_id();
    let mut tags = lock_mutex(&*TAGS)?;
    let tag = match tags.get_tag(guild_id, &name) {
        Ok(tag) => tag,
        Err(err) => return Err(CommandError(err)),
    };

    if !owner_check(msg, &tag) {
        return Err(CommandError("You do not have permission to do that.".to_owned()));
    }

    tags.delete_tag(guild_id, &name);

    check_msg(msg.channel_id.say(&format!("Tag \"{}\" successfully deleted.", name)));
});

// Denies certain tag names from being used as keys.
fn verify_tag_name(name: &str) -> Result<(), String> {
    if name.contains("@everyone") || name.contains("@here") {
        return Err("Tag contains blocked words".to_owned());
    }

    if name.len() > 100 {
        return Err("Tag name limit is 100 characters".to_owned());
    }

    Ok(())
}

fn owner_check(msg: &Message, tag: &Tag) -> bool {
    msg.author.id == tag.owner_id
}

fn get_database_location(guild: Option<GuildId>) -> String {
    guild.map(|g| g.to_string())
        .unwrap_or_else(|| "generic".to_owned())
}
