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
#![deny(warnings)]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(single_char_pattern))]
#![cfg_attr(feature = "nightly", feature(plugin))]

//! Smexybot is a general-purpose [Discord](https://discordapp.com/) bot written
//! in [Rust](https://www.rust-lang.org/). It is built upon the
//! [tumult](https://github.com/indiv0/tumult) bot plugin framework, so Smexybot
//! is easily extensible through the use of plugins.

extern crate discord;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::env;

use discord::{Discord, State};
use discord::model::{ChannelId, Event, Message, UserId};

// The prefix to search for when looking for commands in messages.
const COMMAND_PREFIX: &'static str = "!";

// Denotes which method was used to login to Discord.
#[derive(Clone, Debug, Eq, PartialEq)]
enum LoginMode {
    EmailPassword,
    Token,
}

fn main() {
    // Initialize the `env_logger` to provide logging output.
    env_logger::init().unwrap();

    // Log in to the Discord REST API.
    let (login_mode, discord) = login();

    // Establish a websocket connection to receive events.
    let (mut connection, ready) = discord.connect().expect("Connect failed");
    println!(
        "[Ready] {} is serving {} servers",
        ready.user.username,
        ready.servers.len(),
    );
    let mut state = State::new(ready);

    // Loop over the events recieved by the connection and handle them
    // accordingly, until an unrecoverable error is raised or we receive a quit
    // command.
    loop {
        let event = match connection.recv_event() {
            Ok(event) => event,
            // If we received an error while attempting to receive an event, the
            // most likely cause was a websocket disconnection, so we simply try
            // to reconnect.
            Err(err) => {
                println!("[Warning] Received error: {:?}", err);
                if let discord::Error::WebSocket(..) = err {
                    // Reconnect the websocket.
                    let (new_connection, ready) = discord.connect()
                        .expect("Connect failed");
                    connection = new_connection;
                    state = State::new(ready);
                    println!("[Ready] Reconnected successfully.");
                }
                // If the websocket connection was deliberately closed, the bot
                // should shutdown.
                if let discord::Error::Closed(..) = err {
                    break
                }
                continue
            }
        };
        state.update(&event);

        // If the received event was a message, we want to parse it for
        // commands.
        if let Event::MessageCreate(message) = event {
            trace!("{} says: {}", message.author.name, message.content);

            // If we are running in user mode, ignore any messages not sent by
            // the owner.
            if login_mode == LoginMode::EmailPassword &&
                is_message_author_user(&message, &state.user().id)
            {
                trace!("Ignoring message not from self");
                continue;
            }

            println!("Channel ID: {:?}", message.channel_id);

            // Ignore any messages not directed at us.
            if !message_mentions_user(&message, &state.user().id) &&
                !is_channel_id_private(&state, &message.channel_id)
            {
                trace!("Ignoring message not mentioning us");
                continue;
            }

            // If the message content doesn't contain the command prefix, we
            // ignore it.
            let content = match message.content.find(COMMAND_PREFIX) {
                Some(index) => message.content
                    .chars()
                    .skip(index + COMMAND_PREFIX.len())
                    .collect::<String>(),
                None => {
                    trace!("Ignoring message without command prefix");
                    continue;
                }
            };

            debug!("Received raw command: {}", content);

            // Split the message into its constituent words.
            let mut content = content.split(' ').peekable();

            // Retrieve the command from the message.
            let command = content.peek().map(ToOwned::to_owned);

            if let Some(command) = command {
                debug!("Command: {}", command);
                match command {
                    "test" => {
                        discord.send_message(
                            &message.channel_id,
                            "This is a reply to the test.",
                            "",
                            false,
                        ).ok();
                    },
                    "quit" => {
                        println!("Quitting.");
                        break
                    },
                    _ => {},
                }
            }
        }
    }

    // Attempt to logout gracefully.
    discord.logout().expect("Logout failed");
}

// Determines whether or not the provided `channel_id` refers to a private
// channel (i.e., a DM conversation).
fn is_channel_id_private(state: &State, channel_id: &ChannelId) -> bool {
    state.private_channels().iter().any(|channel| channel.id == *channel_id)
}

// Determines whether or not the message is was authored by the user with ID
// `user_id`.
fn is_message_author_user(message: &Message, user_id: &UserId) -> bool {
    message.author.id == *user_id
}

// Determines whether or not the user with ID `user_id` was mentioned in the
// `message`.
fn message_mentions_user(message: &Message, user_id: &UserId) -> bool {
    let is_mentioned: bool = !message.mentions.is_empty() &&
        message.mentions.iter().any(|m| m.id == *user_id);
    trace!("User {:?} mentioned? {}", user_id, is_mentioned);
    is_mentioned
}

// Attempts to login to the Discord REST API via the available `LoginMode`s.
fn login() -> (LoginMode, Discord) {
    debug!("Attempting to login");

    // Attempt to login via bot token.
    if let Ok(bot_token) = env::var("DISCORD_BOT_TOKEN") {
        debug!("Attempting bot token login");
        return (
            LoginMode::Token,
            Discord::from_bot_token(&bot_token)
                .expect("Bot token login failed"),
        )
    }
    debug!("Skipping bot token login");

    // Attempt to login via email/pass.
    if let (Ok(email), Ok(pass)) = (
        env::var("DISCORD_EMAIL"),
        env::var("DISCORD_PASSWORD"),
    ) {
        debug!("Attempting email/pass login");
        return (
            LoginMode::EmailPassword,
            Discord::new(&email, &pass).expect("Email/pass login failed"),
        )
    }
    debug!("Skipping email/pass login");

    panic!("No suitable authentication method found");
}
