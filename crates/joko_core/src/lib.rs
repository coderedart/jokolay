pub mod init;
pub mod trace;

pub mod prelude {
    // pub use bitflags::bitflags;
    pub use cap_std::fs::Dir;
    pub use egui;
    pub use enumflags2::{self, bitflags, BitFlags};
    pub use glam::*;
    pub use itertools::*;
    pub use miette;
    pub use miette::{bail, Context, Diagnostic, IntoDiagnostic, Result};
    pub use rayon;
    pub use serde;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{from_reader, from_str, to_string_pretty, to_writer_pretty, Value};
    pub use std::collections::{BTreeMap, BTreeSet};
    pub use std::sync::Arc;
    pub use tap::{Pipe, Tap};
    pub use thiserror::{self, Error};
    pub use time::OffsetDateTime;
    pub use tracing::{
        debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
    };
    pub use ureq;
    pub use url::Url;
}
