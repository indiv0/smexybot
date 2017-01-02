// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate time;

use self::time::PreciseTime;

command!(ping(context, _message, _args) {
    let start = PreciseTime::now();
    let msg = context.say("0");
    let end = PreciseTime::now();
    if let Ok(mut m) = msg {
        let ms = start.to(end).num_milliseconds();
        let _ = m.edit(&format!("Pong, {} milliseconds", ms), |m| m);
    }
});
