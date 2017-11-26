// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use error::Result;
use serde_json;
use std::collections::HashSet;
use std::fs::File;
use std::io::{ErrorKind, Read};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Name by which the bot is referred to internally (e.g. in debug output).
    pub bot_name: String,
    /// The command prefix for the bot.
    pub command_prefix: String,
    /// The URL for the source of the project.
    pub source_url: String,
    /// The authors to use in author checks for permissions.
    pub owners: HashSet<u64>,
}

impl Config {
    pub fn new(name: Option<&str>) -> Self {
        if let Some(name) = name {
            match Config::load_from_file(name) {
                Ok(config) => {
                    return config;
                },
                Err(err) => warn!("Failed for load config from \"{}\": {}", name, err),
            }
        }

        info!("Using default config");
        Default::default()
    }

    pub fn load_from_file(name: &str) -> Result<Self> {
        let mut file = match File::open(name) {
            Ok(file) => file,
            // If no file is present, assume this is a fresh config.
            Err(ref err) if err.kind() == ErrorKind::NotFound => return Ok(Default::default()),
            Err(_) => panic!("Failed to open file: {}", name),
        };
        let mut config = String::new();
        file.read_to_string(&mut config)
            .expect(&format!("Failed to read from file: {}", name));
        let config = serde_json::from_str(&config).expect("Failed to deserialize Config");
        info!("Loaded config from: \"{}\"", name);

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            bot_name: "smexybot".to_owned(),
            command_prefix: ";".to_owned(),
            owners: HashSet::new(),
            source_url: "https://github.com/indiv0/smexybot".to_owned(),
        }
    }
}
