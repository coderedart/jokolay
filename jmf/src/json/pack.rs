use crate::json::author::Author;
use crate::json::category::{Cat, CatTree};
use crate::json::marker::Marker;
use crate::json::trail::{TBinDescription, Trail};
use crate::json::{Dirty, ImageDescription};
use serde::*;
use serde_with::*;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FullPack {
    pub pack: Pack,
    pub pack_data: PackData,
    /// tells us whether this has been changed since loading i.e. whether this needs to be saved to file
    #[serde(skip)]
    pub dirty: Dirty,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Pack {
    pub pack_description: PackDescription,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub images_descriptions: BTreeMap<u16, ImageDescription>,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tbins_descriptions: BTreeMap<u16, TBinDescription>,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub cats: BTreeMap<u16, Cat>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cat_tree: Vec<CatTree>,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub markers: BTreeMap<u32, Marker>,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub trails: BTreeMap<u32, Trail>,
}

/// This contains all the images and Tbin files referred to by their ID
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PackData {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub images: BTreeMap<u16, Vec<u8>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tbins: BTreeMap<u16, Vec<[f32; 3]>>,
}

/// Information about the Pack itself. purely informational, not used anywhere
/// All fields are optional
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackDescription {
    /// name of the pack
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Url to the Pack's website
    #[serde(skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// the git repository link. useful if we want to use Git as update mechanism
    #[serde(skip_serializing_if = "String::is_empty")]
    pub git: String,
    /// Authors of the Pack. use this for the "Primary" maintainers of the pack. Contributors can be added to the Category Description Authors field
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub authors: BTreeMap<u16, Author>,

}


impl Pack {
    #[tracing::instrument(skip(self))]
    pub fn save_to_folder_multiple_files(&self, pack_dir: &Path, dirty: &Dirty) -> color_eyre::Result<()> {

        if dirty.pack_desc  {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("pack_desc.json"))?),
                &self.pack_description,
            )?;
        }
        if dirty.image_desc {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("image_desc.json"))?),
                &self.images_descriptions,
            )?;
        }
        if dirty.tbin_desc {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("tbin_desc.json"))?),
                &self.tbins_descriptions,
            )?;
        }
        if dirty.cat_desc {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("cat_desc.json"))?),
                &self.cats,
            )?;
        }
        if dirty.cat_tree {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("cat_tree.json"))?),
                &self.cat_tree,
            )?;
        }
        let markers_dir = pack_dir.join("markers");
        std::fs::create_dir_all(&markers_dir)?;
        let trails_dir = pack_dir.join("trails");
        std::fs::create_dir_all(&trails_dir)?;
        for map_id in dirty.markers.iter().copied() {
            // shift the marker id to higher bytes.
            let start_marker_id = (map_id as u32) << 16;
            let end_marker_id =  start_marker_id + u16::MAX as u32;
            let present_map_markers: BTreeMap<_, _> = self.markers.range(start_marker_id..=end_marker_id).collect();
            let map_markers_path = markers_dir.join(&format!("{map_id}.json"));
            if present_map_markers.is_empty() {
                if  map_markers_path.exists() {
                    std::fs::remove_file(&map_markers_path)?;
                }
            } else {
                serde_json::to_writer_pretty(
                    std::io::BufWriter::new(std::fs::File::create(&map_markers_path)?),
                    &present_map_markers,
                )?;
            }

        }
        for map_id in dirty.trails.iter().copied() {
            // shift the marker id to higher bytes.
            let start_trail_id = (map_id as u32) << 16;
            let end_trail_id =  start_trail_id + u16::MAX as u32;
            let present_map_trails: BTreeMap<_, _> = self.markers.range(start_trail_id..=end_trail_id).collect();
            let map_trails_path = trails_dir.join(&format!("{map_id}.json"));
            if present_map_trails.is_empty() {
                if  map_trails_path.exists() {
                    std::fs::remove_file(&map_trails_path)?;
                }
            } else {
                serde_json::to_writer_pretty(
                    std::io::BufWriter::new(std::fs::File::create(&map_trails_path)?),
                    &present_map_trails,
                )?;
            }

        }
        Ok(())
    }
}

impl PackData {
    pub fn save_to_folder_multiple_files(&self, pack_dir: &Path, dirty: &Dirty) -> color_eyre::Result<()> {
        let images_dir = pack_dir.join("images");
        std::fs::create_dir_all(&images_dir)?;
        let tbins_dir = pack_dir.join("tbins");
        std::fs::create_dir_all(&tbins_dir)?;

        for image_id in dirty.images.iter() {
            let image_path = images_dir.join(&format!("{image_id}.png"));
            if let Some(image_bytes) = self.images.get(image_id) {
                std::fs::File::create(&image_path)?.write_all(image_bytes.as_slice())?;
            } else if image_path.exists() {
                std::fs::remove_file(&image_path)?;
            }
        }
        for tbin_id in dirty.tbins.iter() {
            let tbin_path = tbins_dir.join(&format!("{tbin_id}.tbin"));
            if let Some(tbin_bytes) = self.tbins.get(tbin_id) {
                std::fs::File::create(&tbin_path)?.write_all(bytemuck::cast_slice(tbin_bytes.as_slice()))?;
            } else if tbin_path.exists() {
                std::fs::remove_file(&tbin_path)?;
            }
        }
        Ok(())
    }

}

impl FullPack {
    pub fn save_to_folder_multiple_files(&mut self, pack_dir: &Path, full_save: bool) -> color_eyre::Result<()> {
        if full_save {
            self.dirty = Dirty::full_from_pack(self);
        }
        self.pack.save_to_folder_multiple_files(pack_dir, &self.dirty)?;
        self.pack_data.save_to_folder_multiple_files(pack_dir, &self.dirty)?;
        self.dirty = Dirty::default();
        Ok(())
    }
}