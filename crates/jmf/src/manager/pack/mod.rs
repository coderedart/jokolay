mod archive;
pub mod category;
pub mod marker;
pub mod trail;
pub mod xml;

use crate::{
    is_default,
    manager::pack::{category::CategoryMenu, marker::Marker, trail::Trail},
};
use serde::{Deserialize, Serialize};
use serde_with::*;
use std::{collections::BTreeMap};
use camino::{Utf8Path, Utf8PathBuf};

pub const MARKER_PNG: &[u8] = include_bytes!("../../../assets/marker.png");
pub const TRAIL_PNG: &[u8] = include_bytes!("../../../assets/trail.png");




#[derive(Default, Debug)]
pub struct Pack {
    category_menu: CategoryMenu,
    maps: BTreeMap<u16, MapData>,
    textures: BTreeMap<String, Vec<u8>>,
    trls: BTreeMap<String, Trl>,
}

#[derive(Debug, Default)]
pub struct Trl {
    map_id: u16,
    version: u32,
    nodes: Vec<[f32; 3]>
}
impl Trl {
    pub fn new(map_id: u16, version: u32, nodes: Vec<[f32; 3]>) -> Self {
        Self {
            map_id,
            version,
            nodes,
        }
    }
}

impl Pack {
    pub fn insert_texture(&mut self, name: String, data: Vec<u8>) {
        self.textures.insert(name, data);
    }
    pub fn get_texture(&self, name: &str) -> Option<&[u8]> {
        self.textures.get(name).map(|v| v.as_slice())
    }
    pub fn get_trl(&self, name: &str) -> Option<&Trl> {
        self.trls.get(name)
    }
    pub fn insert_trl(&mut self, name: String, data: Trl) {
        self.trls.insert(name, data);
    }
    pub fn new_category(&mut self, path: &Utf8Path) {
        
    }
    pub fn remove_texture(&mut self, name: &str) {
        self.textures.remove(name);
    }
    pub fn remove_trl(&mut self, name: &str) {
        self.trls.remove(name);
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
struct MapData {
    #[serde(skip_serializing_if = "is_default")]
    markers: Vec<Marker>,
    #[serde(skip_serializing_if = "is_default")]
    trails: Vec<Trail>,
}
