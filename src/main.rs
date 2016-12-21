// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(missing_docs)]
#![deny(non_camel_case_types)]
#![cfg_attr(feature = "nightly", feature(proc_macro))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![warn(missing_copy_implementations,
        missing_debug_implementations,
        missing_docs,
        trivial_numeric_casts,
        unsafe_code,
        unused_extern_crates,
        unused_import_braces,
        unused_qualifications)]
#![cfg_attr(feature="clippy", warn(cast_possible_truncation))]
#![cfg_attr(feature="clippy", warn(cast_possible_wrap))]
#![cfg_attr(feature="clippy", warn(cast_precision_loss))]
#![cfg_attr(feature="clippy", warn(cast_sign_loss))]
#![cfg_attr(feature="clippy", warn(mut_mut))]
// This allows us to use `unwrap` on `Option` values when compiling in test mode
// (because using it in tests is idiomatic).
#![cfg_attr(all(not(test), feature="clippy"), warn(result_unwrap_used))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]

//! Smexybot is a general-purpose [Discord](https://discordapp.com/) bot written
//! in [Rust](https://www.rust-lang.org/). It is built upon the
//! [serenity.rs](https://github.com/zeyla/serenity.rs) Discord API.

extern crate env_logger;
extern crate hyper;
#[cfg(any(feature = "roll", feature = "wolfram", feature = "xkcd"))]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rand;
extern crate serde;
#[cfg(feature = "nightly")]
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serenity;
extern crate url;

mod command;
mod error;
mod util;

use std::env;

use serenity::Client;
use serenity::client::{Context, LoginType};
use serenity::ext::framework::Framework;
use serenity::model::Message;

use util::check_msg;

// The prefix to search for when looking for commands in messages.
const COMMAND_PREFIX: &'static str = "!";
// The ID of the author of the bot.
const AUTHOR_ID: u64 = 117530756263182344;

fn main() {
    // Initialize the `env_logger` to provide logging output.
    env_logger::init().expect("Failed to initialize env_logger");

    // Create a client for a user.
    let (_, mut client) = login();

    client.on_ready(|_context, ready| {
        println!(
            "[Ready] {} is serving {} guilds!",
            ready.user.name,
            ready.guilds.len(),
        );
    });

    client.with_framework(build_framework);

    if let Err(err) = client.start() {
        error!("Client error: {:?}", err);
    }
}

// Configures the `Framework` used by serenity, and registers the handlers for
// any enabled commands.
fn build_framework(framework: Framework) -> Framework {
    let mut framework = framework.configure(|c| c
        .prefix(COMMAND_PREFIX))
    .before(|_context, message, command_name| {
        debug!(
            "Got command '{}' from user '{}'",
            command_name,
            message.author.name,
        );

        true
    })
    .after(|context, _message, command_name, error| {
        if let Err(err) = error {
            check_msg(context.say(&err));
        } else {
            debug!("Processed command '{}'", command_name);
        }
    });

    #[cfg(feature = "fuyu")]
    {
        framework = framework.on("fuyu", command::fuyu::handler);
    }
    #[cfg(feature = "ping")]
    {
        framework = framework.command("ping", |c| c
            .check(owner_check)
            .exec_str("Pong!")
        );
    }
    #[cfg(feature = "roll")]
    {
        framework = framework.on("roll", command::roll::handler);
    }
    #[cfg(feature = "tag")]
    {
        framework = framework.on("tag", command::tag::handler);
    }
    #[cfg(feature = "wolfram")]
    {
        framework = framework.on("wolfram", command::wolfram_alpha::handler);
    }
    #[cfg(feature = "xkcd")]
    {
        framework = framework.on("xkcd", command::xkcd::handler);
    }

    framework
}

fn owner_check(_: &Context, message: &Message) -> bool {
    message.author.id == AUTHOR_ID
}

// Creates a `Client`.
fn login() -> (LoginType, Client) {
    debug!("Attempting to login");

    if let Ok(bot_token) = env::var("DISCORD_BOT_TOKEN") {
        debug!("Performing bot token login");
        return (
            LoginType::Bot,
            Client::login_bot(&bot_token),
        )
    }
    debug!("Skipping bot token login");

    if let Ok(user_token) = env::var("DISCORD_USER_TOKEN") {
        debug!("Performing user token login");
        return (
            LoginType::User,
            Client::login_user(&user_token),
        )
    }
    debug!("Skipping user token login");

    panic!("No suitable authentication method found");
}
