// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Duration, Utc};
use hyper::Client;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use rand::{self, Rng};
use serenity::Result as SerenityResult;
use serenity::model::Message;
use serenity::utils::Colour;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Mutex, MutexGuard};
use tokio_core::reactor::Core;

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
pub fn merge<K: Hash + Eq, V>(mut first: HashMap<K, V>, second: HashMap<K, V>) -> HashMap<K, V>
    where K: Eq + Hash,
{
    for (key, value) in second {
        first.insert(key, value);
    }
    first
}

/// Returns the specified `DateTime<Utc>` as a Discord-compatible ISO 8601
/// `String`.
#[inline]
pub fn timestamp_to_string(timestamp: &DateTime<Utc>) -> String {
    format!("{}", timestamp.format("%Y-%m-%dT%H:%M:%SZ"))
}

/// Returns the specified `Duration` as a `String`.
///
/// Formats the string as "D days H hours M minutes S seconds" if `brief` was
/// set to `false`, and formats it as "Dd Hh Mm Ss" otherwise.
///
/// Days are not added to the resulting string unless applicable.
#[inline]
pub fn duration_to_string(duration: &Duration, brief: bool) -> String {
    let (hours, remainder) = divmod(duration.num_seconds(), 3600);
    let (minutes, seconds) = divmod(remainder, 60);
    let (days, hours) = divmod(hours, 24);

    if brief {
        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else {
            format!("{}h {}m {}s", hours, minutes, seconds)
        }
    } else if days > 0 {
        format!("{} days {} hours {} minutes {} seconds", days, hours, minutes, seconds)
    } else {
        format!("{} hours {} minutes {} seconds", hours, minutes, seconds)
    }
}

/// Converts an error which implements the `Debug` trait into a `String`.
#[inline]
pub fn stringify<E>(error: &E) -> String
    where E: Debug + Error,
{
    format!("Error: {:?}", error)
}

/// Attempts to lock the provided `Mutex`, returning a user-facing error message
/// if the lock failed with a `PoisonError<T>`.
#[inline]
pub fn lock_mutex<T>(mutex: &Mutex<T>) -> Result<MutexGuard<T>, String> {
    mutex.lock().map_err(|_| "Error: an internal error occurred, please report this.".to_owned())
}

/// Initialize a Tokio reactor `Core` and use it to create a TLS hyper `Client`,
///
/// Returns a Tokio `Core` and a hyper `Client`.
pub fn new_core_and_client() -> (Core, Client<HttpsConnector<HttpConnector>>) {
    let core = Core::new().expect("Failed to create Tokio Core");
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).expect("Failed to create HttpsConnector"))
        .build(&core.handle());

    (core, client)
}

/// Take two numbers as arguments and return a pair of numbers consisting of
/// their quotient and remainder when using long division.
fn divmod(a: i64, b: i64) -> (i64, i64) {
    (a / b, a % b)
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use super::duration_to_string;

    #[test]
    fn test_duration_to_string() {
        assert_eq!(duration_to_string(&Duration::seconds(0), true), "0h 0m 0s");
        assert_eq!(duration_to_string(&Duration::seconds(1), true), "0h 0m 1s");
        assert_eq!(duration_to_string(&Duration::seconds(60), true), "0h 1m 0s");
        assert_eq!(duration_to_string(&Duration::seconds(60 * 60), false), "1 hours 0 minutes 0 seconds");
        assert_eq!(duration_to_string(&Duration::seconds(60 * 60 * 24), false), "1 days 0 hours 0 minutes 0 seconds");
        assert_eq!(duration_to_string(&Duration::seconds(60 * 60 * 24 * 365), false), "365 days 0 hours 0 minutes 0 seconds");
    }
}
