// Copyright (c) 2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod json_store;

pub use self::json_store::JsonFileStore;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub trait Store<K, V> {
    fn get<Q>(&self, key: &Q) -> Option<&V> where Q: Borrow<K>;
    fn insert(&mut self, key: K, value: V);
    fn remove<Q>(&mut self, key: &Q) -> Option<V> where Q: Borrow<K>;
}

impl<K, V> Store<K, V> for HashMap<K, V>
    where K: Eq + Hash,
{
    fn get<Q>(&self, key: &Q) -> Option<&V>
        where Q: Borrow<K>,
    {
        self.get(key.borrow())
    }

    fn insert(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    fn remove<Q>(&mut self, key: &Q) -> Option<V>
        where Q: Borrow<K>,
    {
        self.remove(key.borrow())
    }
}
