extern crate typemap;


use self::typemap::Key;
use std::collections::HashMap;

pub struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}
