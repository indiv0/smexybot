// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides a command which allows a user to request and view XKCD comics.

use error::Result;
use futures::{future, Future, Stream};
use hyper::Uri;
use hyper::client::{Client, Connect, HttpConnector};
use hyper_tls::HttpsConnector;
use regex::Regex;
use serde_json;
use std::env;
use std::str::{self, FromStr};
use tokio_core::reactor::Core;
use url::Url;
use util::{check_msg, new_core_and_client};
use xkcd;

lazy_static! {
    static ref GOOGLE_CSE_URL: Url = "https://www.googleapis.com/customsearch/v1".parse::<Url>()
        .expect("Failed to parse URL");
    static ref CSE_API_KEY: String = env::var("GOOGLE_XKCD_CUSTOM_SEARCH_API_KEY")
        .expect("GOOGLE_XKCD_CUSTOM_SEARCH_API_KEY env var not set.");
    static ref CSE_ENGINE_ID: String = env::var("GOOGLE_XKCD_CUSTOM_SEARCH_ENGINE_ID")
        .expect("GOOGLE_XKCD_CUSTOM_SEARCH_ENGINE_ID env var not set.");
    static ref XKCD_URL_REGEX: Regex = Regex::new(r"^https://xkcd.com/(\d*)")
        .expect("Failed to create regex");
}

#[derive(Debug, Deserialize)]
struct CseResponse {
    pub items: Vec<CseItem>,
}

#[derive(Debug, Deserialize)]
struct CseItem {
    pub link: String,
}

struct XkcdPlugin<C> {
    core: Core,
    hyper_client: Client<C>,
    google_custom_search_api_key: String,
    google_custom_search_engine_id: String,
}

impl XkcdPlugin<HttpsConnector<HttpConnector>> {
    /// Returns a new instance of `XkcdPlugin`.
    fn new(google_custom_search_api_key: String, google_custom_search_engine_id: String) -> Self {
        let (core, client) = new_core_and_client();

        XkcdPlugin {
            core: core,
            hyper_client: client,
            google_custom_search_api_key: google_custom_search_api_key,
            google_custom_search_engine_id: google_custom_search_engine_id,
        }
    }

    fn random(&mut self) -> String {
        debug!("Searching for random comic");
        self.core.run(xkcd::random::random(&self.hyper_client))
            .ok()
            .map(|comic| comic.img.into_string())
            .unwrap_or_else(|| "Failed to retrieve random comic".to_owned())
    }

    fn search(&mut self, args: &[String]) -> String {
        debug!("Searching for comic");
        let query: String = match args.len() {
            0 => return "Missing comic search query".to_owned(),
            _ => args.join(" "),
        };
        trace!("Query: {}", query);

        match query_cse(&self.hyper_client,
                        &mut self.core,
                        &query,
                        &self.google_custom_search_api_key,
                        &self.google_custom_search_engine_id) {
            Ok(res) => {
                let mut comic_urls = res.items
                    .iter()
                    .filter_map(|item| XKCD_URL_REGEX.captures_iter(&item.link).next())
                    .filter_map(|capture| capture.get(1));
                match comic_urls.next() {
                    Some(comic_id_str) => {
                        comic_id_str.as_str().parse::<u32>()
                            .ok()
                            .map(|id| {
                                self.core.run(xkcd::comics::get(&self.hyper_client, id))
                                    .ok()
                                    .map(|comic| comic.img.into_string())
                                    .unwrap_or_else(|| format!("Failed to retrieve comic: {}", id))
                            })
                            .unwrap_or_else(|| {
                                format!("Failed to retrieve comic: {}", comic_id_str.as_str())
                            })
                    },
                    None => "No results in query".to_owned(),
                }
            },
            Err(_) => "No matching comic found".to_owned(),
        }
    }

    fn latest_comic(&mut self) -> String {
        debug!("Retrieving latest comic");
        match self.core.run(xkcd::comics::latest(&self.hyper_client)) {
            Ok(comic) => comic.img.into_string(),
            Err(_) => "Failed to retrieve latest comic".to_owned(),
        }
    }
}

command!(xkcd(_ctx, msg, args) {
    let command = args.single::<String>();

    let mut plugin = XkcdPlugin::new(CSE_API_KEY.clone(), CSE_ENGINE_ID.clone());

    let response = match command.ok().as_ref().map(String::as_ref) {
        Some("search") => plugin.search(&args),
        Some("random") => plugin.random(),
        Some(comic_id) => {
            match comic_id.parse() {
                Ok(comic_id) => {
                    match plugin.core.run(xkcd::comics::get(&plugin.hyper_client, comic_id)) {
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
        _ => plugin.latest_comic(),
    };

    check_msg(msg.channel_id.say(response));
});

fn query_cse<C>(client: &Client<C>, core: &mut Core, query: &str, search_api_key: &str, search_engine_id: &str)
        -> Result<CseResponse>
    where C: Connect,
{
    let mut url = GOOGLE_CSE_URL.clone();
    url.query_pairs_mut()
        .clear()
        .append_pair("key", search_api_key)
        .append_pair("cx", search_engine_id)
        .append_pair("q", query);

    let uri = Uri::from_str(url.as_ref()).map_err(From::from);
    let work = future::result(uri)
        .and_then(|uri| client.get(uri))
        .map_err(From::from)
        .and_then(|res| res.body().concat2())
        .map_err(From::from)
        .and_then(|body| {
            str::from_utf8(&body)
                .map_err(From::from)
                .map(|string| string.to_string())
        })
        .and_then(|string| serde_json::from_str(&string).map_err(From::from));

    core.run(work)
}
