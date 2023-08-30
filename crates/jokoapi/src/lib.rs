//! TODO:
//! 1. decide on reqwest or surf or some other http client that will make it easy to use
//! 2. impl the EndPoint trait
//! 3. make sure the client makes it easy to add
//!     1. rate-limiting
//!     2. cache control
//!     3. authorization (bearer token)
//!     4. not duplicate requests in flight
//!     5. wasm support
//!     6. language headers
//!     7. pagination
//!     8. schema
//! for already great implementations look at
//! 1. https://github.com/GW2ToolBelt/GW2APIClient
//! 2. https://github.com/greaka/gw2lib
//! resources:
//! 1. https://wiki.guildwars2.com/wiki/API:API_key
//! 2. https://wiki.guildwars2.com/wiki/API:2
//! 3. https://wiki.guildwars2.com/wiki/API:Main
pub mod end_point;

pub(crate) mod prelude {
    pub type HttpClient = ureq::Agent;
    pub use crate::end_point::EndPoint;
    pub use enumflags2::bitflags;
    pub use miette::{IntoDiagnostic, Result, WrapErr};
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
    pub use std::fmt::Display;
    const API_BASE_URL: &str = "https://api.guildwars2.com";
    pub const API_BASE_V2_URL: &str = const_format::concatcp!(API_BASE_URL, "/v2");
    // https://wiki.guildwars2.com/wiki/API:Changelog
    #[allow(unused)]
    const LATEST_SCHEMA: &str = "2021-04-06T21:00:00.000Z";
}

/*
api->builder
    apikey()
    schema
    endpoints()

enum endpoint
*/
