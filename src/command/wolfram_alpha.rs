// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides the a command which allows a user to query the Wolfram|Alpha API.

extern crate wolfram_alpha;


use hyper::Client;
use self::wolfram_alpha::Error as WolframError;
use self::wolfram_alpha::model::{Pod, QueryResult};
use serenity::client::Context;
use serenity::model::Message;
use serenity::utils::builder::{CreateEmbed, CreateEmbedField};
use std::env;
use std::error::Error as StdError;

use util::{check_msg, random_colour};

lazy_static! {
    static ref PLUGIN: WolframPlugin = {
        let api_app_id = env::var("WOLFRAM_ALPHA_API_APP_ID")
            .expect("WOLFRAM_ALPHA_API_APP_ID env var not set");
        WolframPlugin::new(api_app_id)
    };
}

pub struct WolframPlugin {
    app_id: String,
    hyper_client: Client,
}

impl WolframPlugin {
    /// Returns a new instance of `WolframPlugin`.
    pub fn new(wolfram_alpha_api_app_id: String) -> Self {
        WolframPlugin {
            app_id: wolfram_alpha_api_app_id,
            hyper_client: Client::new(),
        }
    }

    fn query(&self, args: &[String]) -> Result<QueryResult, String> {
        let query = match args.len() {
            0 => return Err("Missing WolframAlpha query".to_owned()),
            _ => args.join(" "),
        };
        trace!("WolframAlpha query: {}", query);

        match wolfram_alpha::query::query(&self.hyper_client, &self.app_id, &query, None) {
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

pub fn handler(context: &Context, message: &Message, args: Vec<String>) -> Result<(), String> {
    // TODO: handle this properly.
    if let Err(err) = context.broadcast_typing(message.channel_id) {
        return Err(format!("{:?}", err));
    }

    match PLUGIN.query(&args) {
        Ok(query_result) => {
            if query_result.success {
                // Format the `QueryResult` into Discord-ready output.
                let colour = random_colour();
                // TODO: get rid of the unwrap here.
                let pods = query_result.pod.unwrap();
                check_msg(context.send_message(
                    message.channel_id,
                    |m| m.embed(|e| format_pods(&pods, e).colour(colour)),
                ));
            } else if let Some(didyoumeans) = query_result.didyoumeans {
                let colour = random_colour();
                check_msg(context.send_message(message.channel_id, |m| {
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
                check_msg(context.send_message(message.channel_id, |m| {
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
                check_msg(context.say("Query was unsuccessful. Perhaps try rewording it?"));
            }
        },
        Err(err) => check_msg(context.say(err.as_ref())),
    }

    Ok(())
}

fn format_pods(pods: &[Pod], embed: CreateEmbed) -> CreateEmbed {
    let mut iter = pods.iter();

    // First result is the interpretation.
    let interpretation = iter.next()
            .unwrap()
            .subpod[0]
        .plaintext
        .clone()
        .unwrap_or("".to_owned());
    let mut embed = embed.title("Input interpretation");
    embed = embed.description(&format!("`{}`", interpretation));

    // TODO: first re-upload the image to an image host, prior the setting the
    // URL.
    if let Some(img) = pods.iter()
        .skip(1)
        .filter_map(|p| p.subpod.iter().filter_map(|s| s.img.clone()).next())
        .next() {
        embed = embed.image(unescape(img.src.as_str()).as_ref());
    }

    // If there is a primary pod, then only format and print that pod.
    if let Some(pod) = pods.iter().find(|p| p.primary == Some(true)) {
        embed = embed.field(|f| format_pod(pod, f));
    } else {
        // Parse all the remaining pods.
        for pod in iter {
            embed = embed.field(|f| format_pod(pod, f.inline(false)));
        }
    }

    embed
}

fn format_pod(pod: &Pod, f: CreateEmbedField) -> CreateEmbedField {
    let f = f.name(pod.title.as_ref());

    trace!("Formatting {} subpods", pod.subpod.len());
    let mut result = String::new();
    for subpod in &pod.subpod {
        let text = match subpod.plaintext {
            // If the text field is empty, add a message indicating as such
            // (discord doesn't like empty/just newline fields).
            Some(ref text) if text == "" || text == "\n" => "[No text]".to_owned(),
            // Grab all the consecutive blobs of text.
            Some(ref text) => {
                trace!("Adding text: {}", text);
                text.to_owned()
            },
            // If there's no text, then there's usually an image.
            _ => {
                match subpod.img {
                    Some(ref img) => {
                        let url = unescape(img.src.as_ref());
                        trace!("Adding img src: {}", url);
                        url
                    },
                    None => break,
                }
            },
        };

        result.push_str(&format!("{}\n", &text));
    }

    // If the output is too long to fit in a single field, truncate it.
    if result.len() > 1024 {
        let truncation_msg = "... (output too long)".to_owned();
        result.truncate(1024 - truncation_msg.len());
        result.push_str(truncation_msg.as_str());
    }

    f.value(result.as_ref())
}

#[inline]
// TODO: find a better way to do this.
fn unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&quot;", "\"")
        .replace("&apos;", "\'")
}
