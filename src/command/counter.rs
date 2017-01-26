// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides functionality for the `counter` command.

use chrono::{DateTime, UTC};
use serenity::client::rest;
use serenity::model::{GuildId, Message, UserId};
use serenity::utils::builder::CreateEmbed;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Mutex;
use store::{JsonFileStore, Store};
use util::{check_msg, lock_mutex, merge, timestamp_to_string};

type Key = String;
type Value = HashMap<String, Counter>;
type CounterStore = JsonFileStore<Key, Value>;

lazy_static! {
    static ref COUNTERS: Mutex<Counters<Key, Value, CounterStore>> = Mutex::new(Counters {
        store: JsonFileStore::new("counters.json".to_owned()),
        _data: PhantomData,
    });
}

#[cfg(feature = "nightly")]
include!("counter.in.rs");

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/counter.rs"));

impl Counter {
    fn new(name: String, owner_id: u64) -> Self {
        Counter {
            name: name,
            count: 0,
            owner_id: owner_id,
            public_edit: true,
            queries: 0,
            location: None,
            created_at: UTC::now(),
            blacklisted_users: HashSet::new(),
            whitelisted_users: HashSet::new(),
        }
    }

    fn as_embed(&self, embed: CreateEmbed) -> CreateEmbed {
        embed.title(&self.name)
            .field(|f| f.name("Owner").value(&format!("<@!{}>", self.owner_id)))
            .field(|f| f.name("Queries").value(&self.queries.to_string()))
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

    fn increment(&mut self) {
        self.count += 1
    }

    fn decrement(&mut self) {
        self.count -= 1
    }
}

#[derive(Debug)]
struct Counters<K, V, S>
    where S: Store<K, V>,
{
    store: S,
    _data: PhantomData<(K, V)>,
}

impl Counters<Key, Value, CounterStore> {
    fn get_possible_counters(&self, guild: Option<GuildId>) -> HashMap<String, Counter> {
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

    fn create_counter(
        &mut self,
        guild: Option<GuildId>,
        name: String,
        counter: Counter
    ) -> Result<(), String> {
        let location = get_database_location(guild);
        let mut database = self.store
            .get(&location)
            .cloned()
            .unwrap_or_else(HashMap::new);
        if database.contains_key(&name) {
            return Err("Counter already exists.".to_owned());
        }

        database.insert(name, counter);
        self.store.insert(location, database);

        Ok(())
    }

    fn get_counter(&self, guild: Option<GuildId>, name: String) -> Result<Counter, String> {
        self.get_possible_counters(guild)
            .get(&name)
            .cloned()
            .ok_or_else(|| "Counter not found".to_owned())
    }

    fn put_counter(&mut self, guild: Option<GuildId>, name: String, counter: Counter) {
        let location = get_database_location(guild);
        let mut database = self.store
            .remove(&location)
            .unwrap_or_else(HashMap::new);
        database.insert(name, counter);
        self.store.insert(location, database);
    }

    fn delete_counter(&mut self, guild: Option<GuildId>, name: String) {
        let location = get_database_location(guild);
        let mut database = self.store
            .remove(&location)
            .unwrap_or_else(HashMap::new);
        database.remove(&name);
        self.store.insert(location, database);
    }
}

command!(counter(context, message, args, first: String) {
    let f = match first.as_ref() {
        "create" => create,
        "info" => info,
        "list" => list,
        "increment" => increment,
        "decrement" => decrement,
        "delete" => delete,
        name => {
            return {
                let guild_id = message.guild_id();

                let lookup = name.to_lowercase();
                let mut counters = lock_mutex(&*COUNTERS)?;
                match counters.get_counter(guild_id, lookup.clone()) {
                    Ok(counter) => {
                        let mut counter = counter.clone();
                        counter.queries += 1;
                        counters.put_counter(guild_id, lookup, counter.clone());
                        check_msg(context.say(&counter.count.to_string()));

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
    let name = name.trim().to_lowercase().to_owned();
    verify_counter_name(&name)?;

    let location = get_database_location(message.guild_id());
    let counter = {
        let mut counter = Counter::new(name.clone(), message.author.id.0);
        counter.location = Some(location.clone());
        counter
    };
    lock_mutex(&*COUNTERS)?.create_counter(message.guild_id(), name.clone(), counter)?;

    check_msg(context.say(&format!("Counter \"{}\" successfully created.", name)));
});

command!(info(context, message, args) {
    let mut args = args.into_iter();

    let name = match args.next() {
        Some(name) => name,
        None => return Err("Please specify a name for the counter to get info on.".to_owned()),
    };

    let name = name.trim().to_lowercase().to_owned();
    let guild_id = message.guild_id();
    let counter = lock_mutex(&*COUNTERS)?.get_counter(guild_id, name)?;

    check_msg(context.send_message(message.channel_id, |m| m.embed(|e| counter.as_embed(e))));
});

command!(list(context, message, _args) {
    let guild_id = message.guild_id();
    let mut counters = {
        let mut counters = lock_mutex(&*COUNTERS)?.get_possible_counters(guild_id);
        let mut counters = counters.drain()
            .map(|(k, _)| k)
            .collect::<Vec<String>>();
        counters.sort();
        counters
    };

    let response = if counters.is_empty() {
        "No counters available.".to_owned()
    } else {
        format!("Available counters: {}", counters.join(", "))
    };
    check_msg(context.say(&response));
});

command!(increment(context, message, args, name: String) {
    let name = name.trim().to_lowercase().to_owned();

    let guild_id = message.guild_id();
    let mut counters = lock_mutex(&*COUNTERS)?;
    let mut counter = match counters.get_counter(guild_id, name.clone()) {
        Ok(counter) => counter,
        Err(err) => return Err(err),
    };

    if !edit_check(message, &counter) {
        return Err("You do not have permission to do that.".to_owned());
    }

    if args.iter().next().is_some() {
        return Err("Unnecessary extra arguments provided".to_owned());
    }

    counter.increment();
    counters.put_counter(guild_id, name.clone(), counter);

    check_msg(context.say(&format!("Counter \"{}\" successfully updated.", name)));
});

command!(decrement(context, message, args, name: String) {
    let name = name.trim().to_lowercase().to_owned();

    let guild_id = message.guild_id();
    let mut counters = lock_mutex(&*COUNTERS)?;
    let mut counter = match counters.get_counter(guild_id, name.clone()) {
        Ok(counter) => counter,
        Err(err) => return Err(err),
    };

    if !edit_check(message, &counter) {
        return Err("You do not have permission to do that.".to_owned());
    }

    if args.iter().next().is_some() {
        return Err("Unnecessary extra arguments provided".to_owned());
    }

    counter.decrement();
    counters.put_counter(guild_id, name.clone(), counter);

    check_msg(context.say(&format!("Counter \"{}\" successfully updated.", name)));
});

command!(delete(context, message, args, name: String) {
    let name = name.trim().to_lowercase().to_owned();

    let guild_id = message.guild_id();
    let mut counters = lock_mutex(&*COUNTERS)?;
    let counter = match counters.get_counter(guild_id, name.clone()) {
        Ok(counter) => counter,
        Err(err) => return Err(err),
    };

    if !owner_check(message, &counter) {
        return Err("You do not have permission to do that.".to_owned());
    }

    counters.delete_counter(guild_id, name.clone());

    check_msg(context.say(&format!("Counter \"{}\" successfully deleted.", name)));
});

// Denies certain counter names from being used as keys.
fn verify_counter_name(name: &str) -> Result<(), String> {
    if name.contains("@everyone") || name.contains("@here") {
        return Err("Counter contains blocked words".to_owned());
    }

    if name.len() > 100 {
        return Err("Counter name limit is 100 characters".to_owned());
    }

    Ok(())
}

fn owner_check(message: &Message, counter: &Counter) -> bool {
    message.author.id == counter.owner_id
}

fn edit_check(message: &Message, counter: &Counter) -> bool {
    let author = message.author.id.0;

    // If the counter is not publically editable, and the message author is not
    // the owner, then they are not allowed to edit it.
    if !counter.public_edit {
        return owner_check(message, counter);
    }

    if !counter.whitelisted_users.is_empty() && !counter.whitelisted_users.contains(&author) {
        // If the whitelist is enabled and the user is not in the whitelist,
        // they are not allowed to edit the counter.
        return false;
    }

    if counter.blacklisted_users.contains(&author) {
        // If the blacklist is enabled and the user is in the blacklist,
        // there are not allowed to edit the counter.
        return false;
    }

    true
}

fn get_database_location(guild: Option<GuildId>) -> String {
    guild.map(|g| g.to_string())
        .unwrap_or_else(|| "generic".to_owned())
}
