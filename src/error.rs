// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use hyper;
use serde_json;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::result::Result as StdResult;
use url;

/// A convenient alias type for results for `smexybot`.
pub type Result<T> = StdResult<T, Error>;

/// Represents errors which occur while using Smexybot.
#[derive(Debug)]
pub enum Error {
    /// A `hyper` crate error.
    Hyper(hyper::Error),
    /// An IO error was encountered.
    Io(io::Error),
    /// A `serde` crate error.
    Serde(serde_json::Error),
    /// Error while parsing a URL.
    UrlParse(url::ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match *self {
            Hyper(ref e) => e.fmt(f),
            Io(ref e) => e.fmt(f),
            Serde(ref e) => e.fmt(f),
            UrlParse(ref e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match *self {
            Hyper(ref e) => e.description(),
            Io(ref e) => e.description(),
            Serde(ref e) => e.description(),
            UrlParse(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        use self::Error::*;

        match *self {
            Hyper(ref e) => e.cause(),
            Io(ref e) => e.cause(),
            Serde(ref e) => e.cause(),
            UrlParse(ref e) => e.cause(),
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error::Hyper(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Serde(error)
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Error {
        Error::UrlParse(error)
    }
}
