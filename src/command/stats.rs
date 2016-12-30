// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate psutil;

use ::{CONFIG, UPTIME};
use chrono::UTC;
use serenity::client::{CACHE, Context};
use serenity::model::{Guild, GuildChannel, Message, UserId};
use util::{check_msg, duration_to_string, timestamp_to_string};

const BYTES_TO_MEGABYTES: f64 = 1f64 / (1024f64 * 1024f64);

pub fn handler(context: &Context, message: &Message, _args: Vec<String>) -> Result<(), String> {
    let current_time = UTC::now();
    let cache = match CACHE.read() {
        Ok(cache) => cache,
        Err(_) => return Err("Failed to lock cache".to_owned()),
    };
    let guilds = cache.guilds
        .values()
        .collect::<Vec<&Guild>>();
    let guilds_count = guilds.len();
    let channels = guilds.iter()
        .flat_map(|g| g.channels.values())
        .collect::<Vec<&GuildChannel>>();
    let channels_count = channels.len();
    let mut user_ids = guilds.iter()
        .flat_map(|g| g.members.keys())
        .collect::<Vec<&UserId>>();
    user_ids.sort();
    user_ids.dedup();
    let users_count = user_ids.len();

    let uptime = current_time - *UPTIME;

    let processes = match psutil::process::all() {
        Ok(processes) => processes,
        Err(_) => return Err("Failed to read process list".to_owned()),
    };
    let process = match processes.iter().find(|p| p.pid == psutil::getpid()) {
        Some(process) => process,
        None => return Err("Failed to retrieve information on process".to_owned()),
    };
    let threads = process.num_threads;
    let memory = match process.memory() {
        Ok(memory) => memory,
        Err(_) => return Err("Failed to retrieve process memory usage".to_owned()),
    };

    let total_mem;
    let resident_mem;
    let shared_mem;
    // TODO: find a way to clean up this attribute.
    #[cfg_attr(feature = "clippy", allow(cast_precision_loss))]
    {
        total_mem = memory.size as f64 * BYTES_TO_MEGABYTES;
        resident_mem = memory.resident as f64 * BYTES_TO_MEGABYTES;
        shared_mem = memory.share as f64 * BYTES_TO_MEGABYTES;
    }

    check_msg(context.send_message(message.channel_id, |m| {
        m.embed(|e| {
            e
            // TODO: extract bot name to config
            .title(&format!("{} stats", CONFIG.bot_name))
            // TODO: official bot server invite link
            // TODO: total/online/unique/unique online members (like R. Danny ?about)
            .field(|f| f.name("Members").value(&users_count.to_string()))
            // TODO: total/text/voice (like R. Danny ?about)
            .field(|f| f.name("Channels").value(&channels_count.to_string()))
            // TODO: change format to Wd Xh Ym Zs
            .field(|f| f.name("Uptime").value(&duration_to_string(&uptime)))
            .field(|f| f.name("Servers").value(&guilds_count.to_string()))
            .field(|f| f.name("Thread Count").value(&threads.to_string()))
            .field(|f| f
                   .name("Memory Usage")
                   .value(&format!(
                           "Total: {:.2} MB\nResident: {:.2} MB\nShared: {:.2} MB",
                           round(total_mem, 2),
                           round(resident_mem, 2),
                           round(shared_mem, 2))))
            // TODO: "Commands run"
            // TODO: memory usage
            // TODO: "Current threads"
            .field(|f| f.name("Source").value(&CONFIG.source_url))
            .timestamp(timestamp_to_string(&current_time))
        })
    }));

    Ok(())
}

/// Rounds a number to the specified decimal precision.
#[inline]
fn round(num: f64, precision: i32) -> f64 {
    let power = 10f64.powi(precision);
    (num * power).round() / power
}
