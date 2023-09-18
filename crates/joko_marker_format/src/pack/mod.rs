mod common;
mod marker;
mod trail;

use std::{collections::BTreeMap, str::FromStr};

use indexmap::IndexMap;

pub use common::*;
pub(crate) use marker::*;
use smol_str::SmolStr;
pub(crate) use trail::*;

#[derive(Default, Debug, Clone)]
pub(crate) struct PackCore {
    pub textures: BTreeMap<RelativePath, Vec<u8>>,
    pub tbins: BTreeMap<RelativePath, TBin>,
    pub categories: IndexMap<String, Category>,
    pub maps: BTreeMap<u32, MapData>,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct MapData {
    pub markers: Vec<Marker>,
    pub trails: Vec<Trail>,
}

#[derive(Debug, Clone)]
pub(crate) struct Category {
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
pub struct RelativePath(SmolStr);
#[allow(unused)]
impl RelativePath {
    pub fn join_str(&self, path: &str) -> Self {
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            return Self(self.0.clone());
        }
        let lower_case = path.to_lowercase();
        if self.0.is_empty() {
            // no need to push `/` if we are empty, as that would make it an absolute path
            return Self(lower_case.into());
        }

        let mut new = self.0.to_string();
        if !self.0.ends_with('/') {
            new.push('/');
        }
        new.push_str(&lower_case);
        Self(new.into())
    }

    pub fn ends_with(&self, ext: &str) -> bool {
        self.0.ends_with(ext)
    }
    pub fn is_png(&self) -> bool {
        self.ends_with(".png")
    }
    pub fn is_tbin(&self) -> bool {
        self.ends_with(".trl")
    }
    pub fn is_xml(&self) -> bool {
        self.ends_with(".xml")
    }
    pub fn is_dir(&self) -> bool {
        self.ends_with("/")
    }
    pub fn parent(&self) -> Option<&str> {
        let path = self.0.trim_end_matches('/');
        if path.is_empty() {
            return None;
        }
        path.rfind('/').map(|index| &path[..=index])
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
impl From<RelativePath> for String {
    fn from(val: RelativePath) -> String {
        val.0.into()
    }
}
impl FromStr for RelativePath {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = s.trim_start_matches('/');
        if path.is_empty() {
            return Ok(Self::default());
        }
        Ok(Self(path.to_lowercase().into()))
    }
}
