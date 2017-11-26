// Copyright (c) 2016-2017 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides the a command which allows a user to query the Wolfram|Alpha API.

use hyper::Client;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use serenity::utils::builder::{CreateEmbed, CreateEmbedField};
use std::env;
use std::error::Error as StdError;
use std::str::FromStr;
use tokio_core::reactor::Core;
use util::{check_msg, new_core_and_client, random_colour, stringify};
use wolfram_alpha::{self, Error as WolframError};
use wolfram_alpha::model::{Pod, QueryResult};

lazy_static! {
    static ref API_APP_ID: String = env::var("WOLFRAM_ALPHA_API_APP_ID")
        .expect("WOLFRAM_ALPHA_API_APP_ID env var not set");
    static ref WOLFRAM_RESULT_SIMPLE_DISPLAY: bool =
        env::var("WOLFRAM_RESULT_SIMPLE_DISPLAY")
            .as_ref()
            .map(|s| &s[..])
            .map(bool::from_str)
            .ok()
            .and_then(Result::ok)
            .unwrap_or(false);
}

pub struct WolframPlugin {
    app_id: String,
    core: Core,
    hyper_client: Client<HttpsConnector<HttpConnector>>,
}

impl WolframPlugin {
    /// Returns a new instance of `WolframPlugin`.
    pub fn new(wolfram_alpha_api_app_id: String) -> Self {
        let (core, client) = new_core_and_client();

        WolframPlugin {
            app_id: wolfram_alpha_api_app_id,
            core,
            hyper_client: client,
        }
    }

    fn query(&mut self, args: &[String]) -> Result<QueryResult, String> {
        let query = match args.len() {
            0 => return Err("Missing WolframAlpha query".to_owned()),
            _ => args.join(" "),
        };
        trace!("WolframAlpha query: {}", query);

        let res = self.core.run(wolfram_alpha::query::query(&self.hyper_client, &self.app_id, &query, None));
        match res {
            Ok(query_result) => Ok(query_result),
            Err(e) => {
                let description = match e {
                    WolframError::Xml(_) => "failed to parse response",
                    _ => e.description(),
                };
                Err(format!("Failed to query WolframAlpha: {}", description))
            },
        }
    }
}

command!(wolfram(_ctx, msg, args) {
    msg.channel_id.broadcast_typing().as_ref().map_err(stringify)?;

    let mut plugin = WolframPlugin::new(API_APP_ID.clone());

    match plugin.query(&args) {
        Ok(query_result) => {
            if query_result.success {
                // Format the `QueryResult` into Discord-ready output.
                let colour = random_colour();
                let pods = query_result.pod
                    .ok_or_else(|| "Result did not contain any parsable information")?;
                check_msg(msg.channel_id.send_message(
                    |m| m.embed(|e| format_pods(&pods, e, *WOLFRAM_RESULT_SIMPLE_DISPLAY).colour(colour)),
                ));
            } else if let Some(didyoumeans) = query_result.didyoumeans {
                let colour = random_colour();
                check_msg(msg.channel_id.send_message(|m| {
                    m.embed(|e| {
                        e.title("Query unsuccessful.")
                            .colour(colour)
                            .field(|f| {
                                let field = f.name("Did you mean:");

                                let mut description = String::new();
                                for item in &didyoumeans.didyoumean {
                                    description.push_str(&format!("* {}\n", item.value));
                                }
                                field.value(description.as_str())
                            })
                    })
                }));
            } else if let Some(error) = query_result.error {
                let colour = random_colour();
                check_msg(msg.channel_id.send_message(|m| {
                    m.embed(|e| {
                        e.title("Wolfram|Alpha returned an error.")
                            .colour(colour)
                            .field(|f| {
                                let field = f.name("Error");

                                let description = format!(
                                    "Code: {}\nMessage: {}",
                                    error.code,
                                    error.msg,
                                );
                                field.value(description.as_str())
                            })
                    })
                }));
            } else {
                check_msg(msg.channel_id.say("Query was unsuccessful. Perhaps try rewording it?"));
            }
        },
        Err(err) => check_msg(msg.channel_id.say(err)),
    }
});

/// Formats pods to be displayed in a Discord embed.
///
/// If `simple` is set to `true`, the embed will only include the primary pod.
fn format_pods(pods: &[Pod], embed: CreateEmbed, simple: bool) -> CreateEmbed {
    let mut iter = pods.iter();

    // First result is the interpretation.
    let interpretation = iter.next()
            .unwrap()
            .subpod[0]
        .plaintext
        .clone()
        .unwrap_or_else(String::new);
    let mut embed = embed.title("Input interpretation");
    embed = embed.description(&format!("`{}`", interpretation));

    if let Some(img) = pods.iter()
        .skip(1)
        .filter_map(|p|
            p.subpod.iter()
                .filter(|s| {
                    // Find all subpods that have a blank "plaintext" field.
                    // This usually indicates they have an image which should be
                    // displayed instead.
                    match s.plaintext {
                        Some(ref text) if text == "" || text == "\n" => true,
                        _ => false,
                    }
                })
                .filter_map(|s| s.img.clone())
                .next()
        )
        .next()
    {
        embed = embed.image(unescape(img.src.as_str()).as_ref());
    }

    // If there is a primary pod, and we only want a simple display, then only
    // format and print that pod.
    if simple {
        if let Some(pod) = pods.iter().find(|p| p.primary == Some(true)) {
            return embed.field(|f| format_pod(pod, f));
        }
    }

    // Parse all the remaining pods.
    pods.iter().fold(embed, |embed, pod| {
        embed.field(|f| format_pod(pod, f))
    })
}

fn format_pod(pod: &Pod, f: CreateEmbedField) -> CreateEmbedField {
    let f = f.name(&pod.title);

    trace!("Formatting {} subpods", pod.subpod.len());
    let mut result = String::new();
    for subpod in &pod.subpod {
        let text = match subpod.plaintext {
            // If the text field exists, but is blank, we likely have an image
            // we should display here instead (and Discord doesn't like
            // empty/just newline fields in embeds anyways).
            // However, image selection is done outside of this function, so we
            // just ignore it.
            Some(ref text) if text == "" || text == "\n" => {
                "[No text]".to_owned()
            },
            // Grab all the consecutive blobs of text.
            Some(ref text) => {
                trace!("Adding text: {}", text);
                text.to_owned()
            },
            // If there was no plaintext field and no image, we don't display
            // anything.
            None => break,
        };

        result.push_str(&format!("{}\n", &text));
    }

    // If the output is too long to fit in a single field, truncate it.
    if result.len() > 1024 {
        let truncation_msg = "... (output too long)".to_owned();
        result.truncate(1024 - truncation_msg.len());
        result.push_str(truncation_msg.as_str());
    }

    f.value(result)
}

#[inline]
fn unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&quot;", "\"")
        .replace("&apos;", "\'")
}
