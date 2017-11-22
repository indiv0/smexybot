// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ::{CONFIG, UPTIME};
use chrono::{Duration, Utc};
use psutil;
use serenity::client::CACHE;
use serenity::framework::standard::CommandError;
use serenity::model::{Guild, UserId};
use std::sync::{Arc, RwLock};
use util::{check_msg, duration_to_string, timestamp_to_string};

const BYTES_TO_MEGABYTES: f64 = 1f64 / (1024f64 * 1024f64);

command!(stats(_ctx, msg, _args) {
    let current_time = Utc::now();

    let cache = match CACHE.read() {
        Ok(cache) => cache,
        Err(why) => {
            debug!("Failed to lock cache: {}", why);
            return Err(CommandError("An internal error occurred".to_owned()));
        },
    };
    let guilds = cache.guilds
        .values()
        .collect::<Vec<&Arc<RwLock<Guild>>>>();
    let guilds_count = guilds.len();
    let mut channels_count = 0;
    let mut user_ids: Vec<UserId> = Vec::new();
    for guild in guilds {
        let guild = guild.read().unwrap();

        channels_count += guild.channels.len();

        let mut users = guild.members.keys().cloned().collect::<Vec<UserId>>();
        user_ids.append(&mut users.clone());
    }
    user_ids.sort();
    user_ids.dedup();
    let users_count = user_ids.len();

    let bot_uptime = current_time.signed_duration_since(*UPTIME);
    let server_uptime = psutil::system::uptime();

    let processes = match psutil::process::all() {
        Ok(processes) => processes,
        Err(why) => {
            debug!("Failed to read process list: {}", why);
            return Err(CommandError("Failed to read process list".to_owned()));
        },
    };
    let process = match processes.iter().find(|p| p.pid == psutil::getpid()) {
        Some(process) => process,
        None => return Err(CommandError("Failed to retrieve information on process".to_owned())),
    };
    let threads = process.num_threads;
    let memory = match process.memory() {
        Ok(memory) => memory,
        Err(why) => {
            debug!("Failed to retrieve process memory usage: {}", why);
            return Err(CommandError("Failed to retrieve process memory usage".to_owned()));
        },
    };

    let total_mem;
    let resident_mem;
    let shared_mem;
    #[cfg_attr(feature = "clippy", allow(cast_precision_loss))]
    {
        total_mem = memory.size as f64 * BYTES_TO_MEGABYTES;
        resident_mem = memory.resident as f64 * BYTES_TO_MEGABYTES;
        shared_mem = memory.share as f64 * BYTES_TO_MEGABYTES;
    }

    check_msg(msg.channel_id.send_message(|m| {
        m.embed(|e| {
            e.title(&format!("{} stats", CONFIG.bot_name))
                .field(|f| f.name("Members").value(&users_count.to_string()))
                .field(|f| f.name("Channels").value(&channels_count.to_string()))
                .field(|f| f.name("Uptime").value(
                        &format!("Bot: {}\nServer: {}",
                            duration_to_string(&bot_uptime, true),
                            duration_to_string(&Duration::seconds(server_uptime as i64), true),
                        )
                ))
                .field(|f| f.name("Servers").value(&guilds_count.to_string()))
                .field(|f| f.name("Thread Count").value(&threads.to_string()))
                .field(|f| {
                    f.name("Memory Usage")
                        .value(&format!("Total: {:.2} MB\nResident: {:.2} MB\nShared: {:.2} MB",
                                        round(total_mem, 2),
                                        round(resident_mem, 2),
                                        round(shared_mem, 2)))
                })
                .field(|f| f.name("Source").value(&CONFIG.source_url))
                .timestamp(timestamp_to_string(&current_time))
        })
    }));
});

/// Rounds a number to the specified decimal precision.
#[inline]
fn round(num: f64, precision: i32) -> f64 {
    let power = 10f64.powi(precision);
    (num * power).round() / power
}
