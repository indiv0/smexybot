// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Duration, UTC};
use rand::{self, Rng};
use serenity::Result as SerenityResult;
use serenity::model::Message;
use serenity::utils::Colour;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;

/// Takes a `Vec<T>` and splits it into a head and a tail.
#[inline]
pub fn split_list<T>(list: Vec<T>) -> (Option<T>, Vec<T>) {
    let mut iter = list.into_iter();
    let head = iter.next();
    let tail = iter.collect();

    (head, tail)
}

/// Checks that a message successfully sent; if not, then logs why.
#[inline]
pub fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}

/// Generates a random RGB colour.
#[inline]
pub fn random_colour() -> Colour {
    let mut rng = rand::thread_rng();
    Colour::new(rng.gen_range::<u32>(0, 0xFFFFFF + 1))
}

/// Takes two `HashMap`s, merges them together, and returns the result.
#[inline]
pub fn merge<K: Hash + Eq, V>(first: HashMap<K, V>, second: HashMap<K, V>) -> HashMap<K, V>
    where K: Eq + Hash,
{
    let mut merged = HashMap::new();
    for (key, value) in first {
        merged.insert(key, value);
    }
    for (key, value) in second {
        merged.insert(key, value);
    }
    merged
}

/// Returns the specified `DateTime<UTC>` as a Discord-compatible ISO 8601
/// `String`.
#[inline]
pub fn timestamp_to_string(timestamp: &DateTime<UTC>) -> String {
    format!("{}", timestamp.format("%Y-%m-%dT%H:%M:%SZ"))
}

/// Returns the specified `Duration` as a `String` in the format "Wd Xh Ym Zs"
#[inline]
pub fn duration_to_string(duration: &Duration) -> String {
    let days = duration.num_days();
    let hours = if days > 0 {
        duration.num_hours() % 24
    } else {
        duration.num_hours()
    };
    let minutes = if hours > 0 {
        duration.num_minutes() % 60
    } else {
        duration.num_minutes()
    };
    let seconds = if minutes > 0 {
        duration.num_seconds() % 60
    } else {
        duration.num_seconds()
    };
    format!(
        "{}d {}h {}m {}s",
        days,
        hours,
        minutes,
        seconds,
    )
}

/// Converts an error which implements the `Debug` trait into a `String`.
#[inline]
pub fn stringify<E>(error: E) -> String
    where E: Debug + Error,
{
    format!("Error: {:?}", error)
}
