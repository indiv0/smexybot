// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
// Build with the system allocator when profiling, for testing purposes.
#![cfg_attr(feature = "profile", feature(alloc_system, global_allocator, allocator_api))]
// This is because serenity's `command!` macro results in an
// `uncreachable_patterns` error when used in the
// `command!(cmd(_, _, _, arg: T))` form.
#![allow(unreachable_patterns)]
#![warn(missing_copy_implementations,
        missing_debug_implementations,
        missing_docs,
        trivial_casts,
        trivial_numeric_casts,
        unused_extern_crates,
        unused_import_braces)]
#![deny(missing_docs, non_camel_case_types, unsafe_code)]
// Allow unsafe code when profiling.
#![cfg_attr(feature = "profile", allow(unsafe_code))]
#![cfg_attr(feature="clippy", warn(
        cast_possible_truncation,
        cast_possible_wrap,
        cast_precision_loss,
        cast_sign_loss,
        mut_mut,
        wrong_pub_self_convention))]
// This allows us to use `unwrap` on `Option` values when compiling in test mode
// (because using it in tests is idiomatic).
#![cfg_attr(all(not(test), feature="clippy"), warn(result_unwrap_used))]

//! Smexybot is a general-purpose [Discord](https://discordapp.com/) bot written
//! in [Rust](https://www.rust-lang.org/). It is built upon the
//! [serenity.rs](https://github.com/zeyla/serenity.rs) Discord API.

#[cfg(feature = "profile")]
extern crate alloc_system;
extern crate chrono;
extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate psutil;
extern crate rand;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate serenity;
extern crate time;
extern crate tokio_core;
extern crate typemap;
extern crate url;
extern crate uuid;
extern crate wolfram_alpha;
extern crate xkcd;

mod command;
mod config;
mod counter;
mod error;
mod store;
mod util;

#[cfg(feature = "profile")]
use alloc_system::System;
use chrono::{DateTime, Utc};
use config::Config;
use counter::CommandCounter;
use serenity::Client;
use serenity::client::{Context, EventHandler};
use serenity::framework::standard::{help_commands, StandardFramework};
use serenity::model::{Ready, UserId};
use std::collections::HashMap;
use std::env;
use util::{check_msg, timestamp_to_string};

#[cfg(feature = "profile")]
#[global_allocator]
static A: System = System;

lazy_static! {
    static ref CONFIG: Config = Config::new(Some("config.json"));
    static ref UPTIME: DateTime<Utc> = Utc::now();
}

struct Handler;

impl EventHandler for Handler {
    // Set a handler to be called on the `on_ready` event.
    // This is called when a shard is booted, and a READY payload is sent by
    // Discord.
    fn on_ready(&self, _: Context, ready: Ready) {
        let shard_info = if let Some(s) = ready.shard {
            Some(format!("shard {}/{} ", s[0] + 1, s[1]))
        } else {
            None
        };

        info!(
            "Started {}as {}#{}, serving {} guilds",
            shard_info.unwrap_or_else(|| "".to_owned()),
            ready.user.name,
            ready.user.discriminator,
            ready.guilds.len(),
        );
    }
}

fn main() {
    // Initialize the `env_logger` to provide logging output.
    env_logger::init().expect("Failed to initialize env_logger");

    // Initialize the `UPTIME` variable.
    debug!("Initialized at: {}", timestamp_to_string(&*UPTIME));

    debug!("Retrieving token from environment");
    let token = env::var("DISCORD_BOT_TOKEN")
        .expect("Failed to find DISCORD_BOT_TOKEN environment variable");

    // Create a client for a user.
    let mut client = Client::new(&token, Handler);

    {
        let mut data = client.data.lock();
        data.insert::<CommandCounter>(HashMap::default());
    }

    client.with_framework(
        StandardFramework::new()
        .configure(|c| c
            .allow_whitespace(false)
            .on_mention(true)
            .prefix(&CONFIG.command_prefix)
            .owners(CONFIG.owners.iter().map(|id| UserId(*id)).collect()))

        .before(|ctx, msg, command_name| {
            trace!(
                "Got command '{}' from user '{}'",
                command_name,
                msg.author.name,
            );

            // Increment the number of times this command has been run.
            // If the command's name does not exist in the counter, add a
            // default value of 0.
            let mut data = ctx.data.lock();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.to_owned()).or_insert(0);
            *entry += 1;

            // if `before` returns false, command processing doesn't happen.
            true
        })
        .after(|_ctx, msg, command_name, error| {
            if let Err(err) = error {
                check_msg(msg.channel_id.say(&err.0));
            } else {
                trace!("Processed command '{}'", command_name);
            }
        })

        .command("help", |c| c.exec_help(help_commands::plain))

        .command("ping", |c| {
            c.desc("Responds with 'Pong', as well as a latency estimate.")
                .exec(command::ping::ping)
                .owners_only(true)
        })

        .command("roll", |c| c.exec(command::roll::roll))

        .command("stats", |c| c.exec(command::stats::stats))

        .command("tag", |c| c.exec(command::tag::tag))

        .command("wolfram", |c| c.exec(command::wolfram_alpha::wolfram))

        .command("xkcd", |c| c.exec(command::xkcd::xkcd))
    );

    // Start the client with an auto-selected number of shards, and start
    // listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until they reconnect.
    if let Err(err) = client.start_autosharded() {
        error!("Client error: {:?}", err);
    }
}
