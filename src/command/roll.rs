// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides functionality for the `!roll` command.

extern crate regex;

use rand::{self, Rng};
use self::regex::Regex;

use util::check_msg;

lazy_static! {
    static ref DICE_ROLL_REGEX: Regex = Regex::new(r"^(\d*)d(\d*)").unwrap();
}

command!(roll(context, _message, args) {
    const ERROR_MESSAGE: &'static str = "Please specify a roll in the form XdY (e.g. 2d6)";

    trace!("Received roll command with args: {:?}", args);
    let arg = match args.iter().next() {
        Some(arg) => arg,
        None => {
            check_msg(context.say(ERROR_MESSAGE));
            return Ok(());
        },
    };

    let mut captures = DICE_ROLL_REGEX.captures_iter(arg);
    let next_capture = captures.next();
    let (number_of_dice, die_sides) = match next_capture {
        Some(capture) => {
            let number_of_dice = match capture.at(1) {
                Some(number_of_dice) => {
                    match number_of_dice.parse::<u32>() {
                        Ok(number_of_dice) => number_of_dice,
                        _ => {
                            check_msg(context.say(ERROR_MESSAGE));
                            return Ok(());
                        },
                    }
                },
                _ => {
                    check_msg(context.say(ERROR_MESSAGE));
                    return Ok(());
                },
            };

            let die_sides = match capture.at(2) {
                Some(die_sides) => {
                    match die_sides.parse::<u32>() {
                        Ok(0) => {
                            check_msg(context.say("Number of die sides cannot be 0."));
                            return Ok(());
                        },
                        Ok(4294967295) => {
                            check_msg(context.say("Number of die sides is too large"));
                            return Ok(());
                        },
                        Ok(die_sides) => die_sides,
                        _ => {
                            check_msg(context.say(ERROR_MESSAGE));
                            return Ok(());
                        },
                    }
                },
                _ => {
                    check_msg(context.say(ERROR_MESSAGE));
                    return Ok(());
                },
            };

            assert!(die_sides > 0 && die_sides < 4294967295);

            (number_of_dice, die_sides)
        },
        _ => {
            check_msg(context.say(ERROR_MESSAGE));
            return Ok(());
        },
    };

    if number_of_dice == 0 {
        check_msg(context.say("Number of dice cannot be 0"));
        return Ok(());
    }

    let mut rolls = Vec::new();
    let mut rng = rand::thread_rng();
    let mut sum = 0u32;
    for _ in 1..(number_of_dice + 1) {
        let roll = rng.gen_range::<u32>(1, die_sides + 1);
        sum = sum.checked_add(roll).ok_or("Unable to calculate result: sum of rolls too large")?;
        rolls.push(roll);
    }
    let roll_string = rolls.iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(" + ");

    let response = match rolls.len() {
        1 => sum.to_string(),
        _ => format!("{} = {}", roll_string, sum),
    };

    check_msg(context.say(response.as_ref()));
});
