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
use std::collections::BTreeMap;

pub const MARKER_PNG: &[u8] = include_bytes!("../../../assets/marker.png");
pub const TRAIL_PNG: &[u8] = include_bytes!("../../../assets/trail.png");

#[derive(Default, Debug)]
pub struct Pack {
    category_menu: CategoryMenu,
    maps: BTreeMap<u16, MapData>,
    images: BTreeMap<String, Vec<u8>>,
    tbins: BTreeMap<String, Vec<[f32; 3]>>,
}

impl Pack {
    pub fn insert_image(&mut self, name: String, data: Vec<u8>) {
        self.images.insert(name, data);
    }
    pub fn get_image(&self, name: &str) -> Option<&[u8]> {
        self.images.get(name).map(|v| v.as_slice())
    }
    pub fn get_tbin(&self, name: &str) -> Option<&[[f32; 3]]> {
        self.tbins.get(name).map(|v| v.as_slice())
    }
    pub fn insert_tbin(&mut self, name: String, data: Vec<[f32; 3]>) {
        self.tbins.insert(name, data);
    }
    pub fn new_category(&mut self, parent: Option<u16>) {
        self.category_menu.create_child_category(parent);
    }
    pub fn remove_image(&mut self, name: &str) {
        self.images.remove(name);
    }
    pub fn remove_tbin(&mut self, name: &str) {
        self.tbins.remove(name);
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
