mod archive;
pub mod category;
pub mod marker;
pub mod trail;
pub mod xml;

use crate::{
    is_default,
    manager::pack::{category::CategoryMenu, marker::Marker, trail::Trail},
};
use camino::Utf8Path;
use color_eyre::*;
use serde::{Deserialize, Serialize};
use serde_with::*;
use std::{collections::BTreeMap, path::Path};
use walkdir::WalkDir;
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
    nodes: Vec<[f32; 3]>,
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
    pub fn new_category(&mut self, _path: &Utf8Path) {}
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

impl Pack {
    pub fn save_to_directory(&self, root_dir: &std::path::Path) -> color_eyre::Result<()> {
        let textures_dir = root_dir.join("textures");
        let trls_dir = root_dir.join("trls");
        let maps_dir = root_dir.join("maps");
        std::fs::write(
            root_dir.join("cats.json"),
            serde_json::to_string_pretty(&self.category_menu).expect("failed ot deserialize cats"),
        )
        .expect("failed to write cats.json");
        std::fs::create_dir_all(&textures_dir).expect("failed to cratee textrues dir");
        for (name, texture) in self.textures.iter() {
            std::fs::write(textures_dir.join(&format!("{name}.png")), texture)
                .expect("failed to write texture ot file");
        }
        std::fs::create_dir_all(&trls_dir).expect("failed to create trls dir");
        for (name, trl) in self.trls.iter() {
            let mut trl_bytes = vec![];
            trl_bytes.extend_from_slice(&(trl.map_id as u32).to_ne_bytes());
            trl_bytes.extend_from_slice(&trl.version.to_ne_bytes());
            trl_bytes.extend_from_slice(bytemuck::cast_slice(&trl.nodes));
            std::fs::write(trls_dir.join(&format!("{name}.trl")), &trl_bytes)
                .expect("failed to write trl to file");
        }
        std::fs::create_dir_all(&maps_dir).expect("failed to create maps dir");
        for (map_id, map_markers) in self.maps.iter() {
            std::fs::write(
                maps_dir.join(&format!("{map_id}.json")),
                serde_json::to_string_pretty(&map_markers)
                    .expect("failed to deserialize map markers"),
            )
            .expect("failed to write map data to file");
        }
        Ok(())
    }

    pub fn load_from_directory(pack_root: &Path) -> Result<Self> {
        let mut pack = Self::default();
        let cats_path = pack_root.join("cats.json");
        pack.category_menu = serde_json::from_str(std::fs::read_to_string(&cats_path)?.as_str())?;
        for texture_entry in walkdir::WalkDir::new(&pack_root.join("textures")).min_depth(1) {
            let texture_entry = texture_entry?;
            let texture_name = texture_entry
                .file_name()
                .to_str()
                .expect("non utf file name");
            let texture_name = texture_name
                .strip_suffix(".png")
                .expect("failed to strip .png");
            assert!(texture_entry
                .metadata()
                .expect("failed to get metadata")
                .is_file());
            pack.textures.insert(
                texture_name.to_string(),
                std::fs::read(&texture_entry.path())?,
            );
        }

        for map_entry in WalkDir::new(&pack_root.join("maps")).min_depth(1) {
            let map_entry = map_entry?;
            assert!(map_entry
                .metadata()
                .expect("failed to get metadata")
                .is_file());
            pack.maps.insert(
                map_entry
                    .file_name()
                    .to_str()
                    .expect("failed to convert to str")
                    .strip_suffix(".json")
                    .expect("failed to strip .json from map name")
                    .parse()
                    .expect("failed to extract map name"),
                serde_json::from_str(&std::fs::read_to_string(map_entry.path())?)?,
            );
        }
        Ok(pack)
    }
}
