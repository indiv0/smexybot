// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rand::{self, Rng};
use serenity::Result as SerenityResult;
use serenity::model::Message;
use serenity::utils::Colour;

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
