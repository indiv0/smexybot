// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "fuyu")]
pub mod fuyu;
#[cfg(feature = "roll")]
pub mod roll;
#[cfg(feature = "wolfram")]
pub mod wolfram_alpha;
#[cfg(feature = "xkcd")]
pub mod xkcd;
