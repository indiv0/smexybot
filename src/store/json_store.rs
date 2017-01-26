// Copyright (c) 2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::{Deserialize, Serialize};
use serde_json;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::Hash;
use std::io::{ErrorKind, Read, Write};
use store::Store;
use uuid::Uuid;

#[derive(Debug)]
pub struct JsonFileStore<K, V>
    where K: Eq + Hash,
{
    name: String,
    store: HashMap<K, V>,
}

impl<K, V> JsonFileStore<K, V>
    where K: Deserialize + Eq + Hash + Serialize,
          V: Deserialize + Serialize,
{
    fn load(&mut self) {
        let mut file = match File::open(&self.name) {
            Ok(file) => file,
            // If no file is present, assume this is a fresh config.
            Err(ref err) if err.kind() == ErrorKind::NotFound => return,
            Err(_) => panic!("Failed to open file: {}", self.name),
        };
        let mut store = String::new();
        file.read_to_string(&mut store)
            .expect(&format!("Failed to read from file: {}", self.name));
        self.store = serde_json::from_str(&store).expect("Failed to deserialize store");
        debug!("Loaded config from: {}", self.name);
    }

    fn save(&self) {
        let temp = format!("{}-{}.tmp", Uuid::new_v4(), self.name);
        let mut file = File::create(&temp).expect(&format!("Failed to create file: {}", temp));
        file.write_all(serde_json::to_string(&self.store)
                .expect("Failed to serialize Config")
                .as_bytes())
            .expect(&format!("Failed to write to file: {}", temp));

        // Atomically copy the new config.
        fs::rename(temp, &self.name).expect("Failed to write new store");
        trace!("Saved config to: {}", self.name);
    }
}

impl<K, V> JsonFileStore<K, V>
    where K: Deserialize + Eq + Hash + Serialize,
          V: Deserialize + Serialize,
{
    pub fn new(name: String) -> Self {
        let mut store = JsonFileStore {
            name: name,
            store: HashMap::new(),
        };
        store.load();

        store
    }
}

impl<K, V> Store<K, V> for JsonFileStore<K, V>
    where K: Deserialize + Eq + Hash + Serialize,
          V: Deserialize + Serialize,
{
    fn get<Q>(&self, key: &Q) -> Option<&V>
        where Q: Borrow<K>,
    {
        self.store.get(key.borrow())
    }

    fn insert(&mut self, key: K, value: V) {
        self.store.insert(key, value);
        self.save();
    }

    fn remove<Q>(&mut self, key: &Q) -> Option<V>
        where Q: Borrow<K>,
    {
        let res = self.store.remove(key.borrow());
        self.save();
        res
    }
}
