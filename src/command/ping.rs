// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use time::PreciseTime;

command!(ping(_ctx, msg, _args) {
    let start = PreciseTime::now();
    let result = msg.channel_id.say("Pong");
    let end = PreciseTime::now();
    let ping_ms = start.to(end).num_milliseconds();

    if let Ok(mut message) = result {
        let _ = message.edit(|m| m.content(&format!("Pong, {} milliseconds", ping_ms)));
    }
});
