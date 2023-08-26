mod common;
mod marker;
mod trail;

use std::collections::BTreeMap;

use indexmap::IndexMap;

pub const MARKER_PNG: &[u8] = include_bytes!("marker.png");
pub const TRAIL_PNG: &[u8] = include_bytes!("trail.png");

use relative_path::RelativePathBuf;

pub use common::*;
pub use marker::*;
pub use trail::*;

#[derive(Default, Debug)]
pub struct PackCore {
    pub textures: BTreeMap<RelativePathBuf, Vec<u8>>,
    pub tbins: BTreeMap<RelativePathBuf, TBin>,
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
