// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides functionality for the `tag` command.

use chrono::{DateTime, UTC};
use serenity::client::rest;
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

#[cfg(feature = "nightly")]
include!("tag.in.rs");

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/tag.rs"));

impl Tag {
    fn new(name: String, content: String, owner_id: u64) -> Self {
        Tag {
            name: name,
            content: content,
            owner_id: owner_id,
            uses: 0,
            location: None,
            created_at: UTC::now(),
        }
    }

    fn as_embed(&self, embed: CreateEmbed) -> CreateEmbed {
        embed.title(&self.name)
            .field(|f| f.name("Owner").value(&format!("<@!{}>", self.owner_id)))
            .field(|f| f.name("Uses").value(&self.uses.to_string()))
            .author(|a| {
                let owner_id = UserId(self.owner_id);
                let (name, avatar_url) = match owner_id.find() {
                    Some(user) => (user.name.clone(), user.avatar_url()),
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

    fn create_tag(&mut self, guild: Option<GuildId>, name: String, tag: Tag) -> Result<(), String> {
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

    fn get_tag(&self, guild: Option<GuildId>, name: String) -> Result<Tag, String> {
        self.get_possible_tags(guild)
            .get(&name)
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

command!(tag(context, message, args, first: String) {
    let f = match first.as_ref() {
        "create" => create,
        "info" => info,
        "list" => list,
        "edit" => edit,
        "delete" => delete,
        name => {
            return {
                let guild_id = message.guild_id();

                let lookup = name.to_lowercase();
                let mut tags = lock_mutex(&*TAGS)?;
                match tags.get_tag(guild_id, lookup.clone()) {
                    Ok(tag) => {
                        let mut tag = tag.clone();
                        let content = tag.content.clone();
                        tag.uses += 1;
                        tags.put_tag(guild_id, lookup, tag);
                        check_msg(context.say(&content));

                        Ok(())
                    },
                    Err(err) => Err(err),
                }
            };
        },
    };

    // This is necessary because the `command!` macro returns `Ok(())`. Without
    // this match and fall-through, rustc would complain about unreachable code.
    match f(context, message, args.clone()) {
        Ok(()) => {},
        v => return v,
    }
});

command!(create(context, message, args, name: String) {
    let content = if args.is_empty() {
        return Err("Please specify some content for the tag.".to_owned());
    } else {
        args.join(" ")
    };

    let name = name.trim().to_lowercase().to_owned();
    verify_tag_name(&name)?;

    let guild = message.guild_id();
    let location = get_database_location(guild);
    let tag = {
        let mut tag = Tag::new(
            name.clone(),
            content,
            message.author.id.0,
        );
        tag.location = Some(location);
        tag
    };
    lock_mutex(&*TAGS)?.create_tag(guild, name.clone(), tag)?;

    check_msg(context.say(&format!("Tag \"{}\" successfully created.", name)));
});

command!(info(context, message, args, name: String) {
    let name = name.trim().to_lowercase().to_owned();
    let guild_id = message.guild_id();
    let tag = lock_mutex(&*TAGS)?.get_tag(guild_id, name)?;

    check_msg(context.send_message(message.channel_id, |m| m.embed(|e| tag.as_embed(e))));
});

command!(list(context, message, _args) {
    let guild_id = message.guild_id();
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
    check_msg(context.say(&response));
});

command!(edit(context, message, args, name: String) {
    let name = name.trim().to_lowercase().to_owned();

    let guild_id = message.guild_id();
    let mut tags = lock_mutex(&*TAGS)?;
    let mut tag = tags.get_tag(guild_id, name.clone())?;

    if !owner_check(message, &tag) {
        return Err("You do not have permission to do that.".to_owned());
    }

    let content = if args.is_empty() {
        return Err("Please specify some content for the tag.".to_owned());
    } else {
        args.join(" ")
    };

    tag.content = content;
    tags.put_tag(guild_id, name.clone(), tag);

    check_msg(context.say(&format!("Tag \"{}\" successfully updated.", name)));
});

command!(delete(context, message, args, name: String) {
    let name = name.trim().to_lowercase().to_owned();

    let guild_id = message.guild_id();
    let mut tags = lock_mutex(&*TAGS)?;
    let tag = match tags.get_tag(guild_id, name.clone()) {
        Ok(tag) => tag,
        Err(err) => return Err(err),
    };

    if !owner_check(message, &tag) {
        return Err("You do not have permission to do that.".to_owned());
    }

    tags.delete_tag(guild_id, &name);

    check_msg(context.say(&format!("Tag \"{}\" successfully deleted.", name)));
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

fn owner_check(message: &Message, tag: &Tag) -> bool {
    message.author.id == tag.owner_id
}

fn get_database_location(guild: Option<GuildId>) -> String {
    guild.map(|g| g.to_string())
        .unwrap_or_else(|| "generic".to_owned())
}
