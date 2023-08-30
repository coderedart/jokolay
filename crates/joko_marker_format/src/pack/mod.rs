mod common;
mod marker;
mod trail;

use std::collections::BTreeMap;

use indexmap::IndexMap;

pub const MARKER_PNG: &[u8] = include_bytes!("marker.png");
pub const TRAIL_PNG: &[u8] = include_bytes!("trail.png");

pub use common::*;
pub use marker::*;
pub use trail::*;

#[derive(Default, Debug)]
pub struct PackCore {
    pub textures: BTreeMap<RelativePath, Vec<u8>>,
    pub tbins: BTreeMap<RelativePath, TBin>,
    pub categories: IndexMap<String, Category>,
    pub maps: BTreeMap<u32, MapData>,
}

#[derive(Default, Debug)]
pub struct MapData {
    pub markers: Vec<Marker>,
    pub trails: Vec<Trail>,
}

impl PackCore {}

#[derive(Debug)]
pub struct Category {
    pub display_name: String,
    pub separator: bool,
    pub default_enabled: bool,
    pub props: CommonAttributes,
    pub children: IndexMap<String, Category>,
}

/// This newtype is used to represents relative paths in marker packs
/// 1. It won't start with `/` or `C:` like roots, because its a relative path
/// 2. It can be empty to represent current directory
/// 3. No expansion of special characters like  `.` or `..` stuff.
/// 4. It is always lowercase to avoid platform specific quirks.
/// 5. It will use `/` as the path separator.
/// 6. It doesn't mean that the path is valid. It may contain many of the utf-8 characters which are not valid path names on linux/windows
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativePath(String);

impl RelativePath {
    pub fn parse_from_str(path: &str) -> Self {
        if path.is_empty() {
            return Self::default();
        }
        let path = path.trim_start_matches('/');
        Self(path.to_lowercase())
    }
    pub fn join_str(&self, path: &str) -> Self {
        let mut new = self.0.to_string();
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            return Self(new);
        }
        let lower_case = path.to_lowercase();
        if self.0.is_empty() {
            // no need to push `/` if we are empty, as that would make it an absolute path
            return Self(lower_case);
        }
        if !self.0.ends_with('/') {
            new.push('/');
        }
        new.push_str(&lower_case);
        Self(new)
    }

    pub fn ends_with(&self, ext: &str) -> bool {
        self.0.ends_with(ext)
    }
    pub fn parent(&self) -> Option<&str> {
        let path = self.0.trim_end_matches('/');
        if path.is_empty() {
            return None;
        }
        path.rfind('/').map(|index| &path[..index])
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
