// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides a command which allows a user to request and view XKCD comics.

extern crate regex;
extern crate xkcd;


use error::{Error, Result};

use hyper::Url;
use hyper::client::Client;
use self::regex::Regex;
use serde_json;
use serenity::client::Context;
use serenity::model::Message;
use std::env;
use std::io::Read;
use std::result::Result as StdResult;
use util::{check_msg, split_list};

lazy_static! {
    static ref GOOGLE_CUSTOM_SEARCH_URL: Url = "https://www.googleapis.com/customsearch/v1".parse::<Url>()
        .unwrap();
    static ref PLUGIN: XkcdPlugin = {
        let cse_api_key = env::var("GOOGLE_XKCD_CUSTOM_SEARCH_API_KEY")
            .expect("GOOGLE_XKCD_CUSTOM_SEARCH_API_KEY env var not set.");
        let cse_engine_id = env::var("GOOGLE_XKCD_CUSTOM_SEARCH_ENGINE_ID")
            .expect("GOOGLE_XKCD_CUSTOM_SEARCH_ENGINE_ID env var not set.");
        XkcdPlugin::new(cse_api_key, cse_engine_id)
    };
    static ref XKCD_URL_REGEX: Regex = Regex::new(r"^https://xkcd.com/(\d*)").unwrap();
}

#[cfg(feature = "nightly")]
include!("xkcd.in.rs");

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/xkcd.rs"));

struct XkcdPlugin {
    hyper_client: Client,
    google_custom_search_api_key: String,
    google_custom_search_engine_id: String,
}

impl XkcdPlugin {
    /// Returns a new instance of `XkcdPlugin`.
    fn new(google_custom_search_api_key: String, google_custom_search_engine_id: String) -> Self {
        XkcdPlugin {
            hyper_client: Client::new(),
            google_custom_search_api_key: google_custom_search_api_key,
            google_custom_search_engine_id: google_custom_search_engine_id,
        }
    }

    fn random(&self) -> String {
        debug!("Searching for random comic");
        xkcd::random::random(&self.hyper_client)
            .ok()
            .map(|comic| comic.img.into_string())
            .unwrap_or("Failed to retrieve random comic".to_owned())
    }

    fn search(&self, args: &[String]) -> String {
        debug!("Searching for comic");
        let query: String = match args.len() {
            0 => return "Missing comic search query".to_owned(),
            _ => args.join(" "),
        };
        trace!("Query: {}", query);

        match query_cse(&self.hyper_client,
                        &query,
                        &self.google_custom_search_api_key,
                        &self.google_custom_search_engine_id) {
            Ok(res) => {
                let mut comic_urls = res.items
                    .iter()
                    .filter_map(|item| XKCD_URL_REGEX.captures_iter(&item.link).next())
                    .filter_map(|capture| capture.at(1));
                match comic_urls.next() {
                    Some(comic_id_str) => comic_id_str.parse::<u32>()
                        .ok()
                        .map(|id| {
                            xkcd::comics::get(&self.hyper_client, id)
                                .ok()
                                .map(|comic| comic.img.into_string())
                                .unwrap_or_else(|| format!("Failed to retrieve comic: {}", id))
                        })
                        .unwrap_or_else(|| format!("Failed to retrieve comic: {}", comic_id_str)),
                    None => "No results in query".to_owned(),
                }
            },
            Err(_) => "No matching comic found".to_owned(),
        }
    }

    fn latest_comic(&self) -> String {
        debug!("Retrieving latest comic");
        match xkcd::comics::latest(&self.hyper_client) {
            Ok(comic) => comic.img.into_string(),
            Err(_) => "Failed to retrieve latest comic".to_owned(),
        }
    }
}

pub fn handler(context: &Context, _message: &Message, args: Vec<String>) -> StdResult<(), String> {
    let (command, args) = split_list(args);

    let response = match command.as_ref().map(String::as_ref) {
        Some("search") => PLUGIN.search(&args),
        Some("random") => PLUGIN.random(),
        Some(comic_id) => {
            match comic_id.parse() {
                Ok(comic_id) => {
                    match xkcd::comics::get(&PLUGIN.hyper_client, comic_id) {
                        Ok(comic) => comic.img.into_string(),
                        Err(_) => format!("Failed to retrieve comic: {}", comic_id),
                    }
                },
                _ => {
                    "Please provide a valid argument (\"search\", \"random\", or a comic ID)"
                        .to_owned()
                },
            }
        },
        _ => PLUGIN.latest_comic(),
    };

    check_msg(context.say(response.as_ref()));

    Ok(())
}

fn query_cse(
    client: &Client,
    query: &str,
    search_api_key: &str,
    search_engine_id: &str
) -> Result<CseResponse> {
    let mut url = GOOGLE_CUSTOM_SEARCH_URL.clone();
    url.query_pairs_mut()
        .clear()
        .append_pair("key", search_api_key)
        .append_pair("cx", search_engine_id)
        .append_pair("q", query);

    let mut response = try!(client.get(url).send().map_err(Error::from));
    let mut result = String::new();
    try!(response.read_to_string(&mut result).map_err(Error::from));

    serde_json::from_str(&result).map_err(Error::from)
}
