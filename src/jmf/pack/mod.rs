mod types;
pub mod xml;

use intmap::IntMap;
pub use types::*;
pub const MARKER_PNG: &[u8] = include_bytes!("marker.png");
pub const TRAIL_PNG: &[u8] = include_bytes!("trail.png");
use bytecheck::CheckBytes;
use glam::Vec3;
use rkyv::*;
use std::collections::BTreeMap;
#[derive(Debug, Archive, Serialize)]
#[archive_attr(derive(CheckBytes))]
pub struct ZTex {
    pub width: u16,
    pub height: u16,
    pub bytes: Vec<u8>,
}
#[derive(Debug, Archive, Serialize, Clone, Copy)]
#[archive_attr(derive(CheckBytes, Copy, Clone))]
pub struct ZCat {
    pub display_name: u16,
    pub is_separator: bool,
    pub parent_id: u16,
}

#[derive(Debug, Archive, Serialize, Default)]
#[archive_attr(derive(CheckBytes))]
pub struct ZMapData {
    pub markers: Vec<ZMarker>,
    pub trails: Vec<ZTrail>,
}

#[derive(Debug, Archive, Serialize)]
#[archive_attr(derive(CheckBytes))]
pub struct ZMarker {
    pub position: Vec3,
    pub cat: u16,
    pub texture: u16,
}

#[derive(Debug, Archive, Serialize)]
#[archive_attr(derive(CheckBytes))]
pub struct ZTrail {
    pub cat: u16,
    pub texture: u16,
    pub tbin: u16,
}

#[derive(Debug, Archive, Serialize)]
#[archive_attr(derive(CheckBytes))]
pub struct ZPack {
    pub version: String,
    pub timestamp: f64,
    pub textures: Vec<ZTex>,
    pub tbins: Vec<Vec<glam::Vec3>>,
    pub text: Vec<String>,
    pub cats: Vec<ZCat>,
    pub maps: BTreeMap<u16, ZMapData>,
}

pub struct ActivationData {
    pub cats_status: bitvec::vec::BitVec,
    /// the key is marker id. and the value is the reset timestamp i.e. marker is reactivated.
    pub markers_status: IntMap<u32>,
}
