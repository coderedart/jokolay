mod common;
mod marker;
mod trail;

use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;

pub const MARKER_PNG: &[u8] = include_bytes!("marker.png");
pub const TRAIL_PNG: &[u8] = include_bytes!("trail.png");

use joko_core::prelude::OffsetDateTime;
use relative_path::RelativePathBuf;

use uuid::Uuid;

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

pub struct ActivationData {
    /// key is packname
    /// value is a set of category names which are enabled
    pub cats_status: BTreeMap<String, BTreeSet<String>>,
    /// the key is marker guid. value is *when* we can remove this marker from the triggered marker list.
    /// the guids are global across all marker packs and maps
    pub markers_status: BTreeMap<Uuid, OffsetDateTime>,
}
