extern crate typemap;

use std::collections::HashMap;

use self::typemap::Key;

pub struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}
