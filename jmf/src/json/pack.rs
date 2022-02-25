use crate::json::author::Author;
use crate::json::category::{Cat, CatTree};
use crate::json::marker::Marker;
use crate::json::trail::{TBinDescription, Trail};
use crate::json::ImageDescription;
use serde::*;
use serde_with::*;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FullPack {
    pub pack: Pack,
    pub pack_data: PackData,
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
    #[tracing::instrument]
    pub fn save_to_folder(&self, pack_dir: &Path) -> color_eyre::Result<()> {
        serde_json::to_writer_pretty(
            std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("pack_desc.json"))?),
            &self.pack_description,
        )?;
        serde_json::to_writer_pretty(
            std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("image_desc.json"))?),
            &self.images_descriptions,
        )?;
        serde_json::to_writer_pretty(
            std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("tbin_desc.json"))?),
            &self.tbins_descriptions,
        )?;
        serde_json::to_writer_pretty(
            std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("cat_desc.json"))?),
            &self.cats,
        )?;
        serde_json::to_writer_pretty(
            std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("cat_tree.json"))?),
            &self.cat_tree,
        )?;
        serde_json::to_writer_pretty(
            std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("markers.json"))?),
            &self.markers,
        )?;
        serde_json::to_writer_pretty(
            std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("trails.json"))?),
            &self.trails,
        )?;
        std::fs::create_dir_all(pack_dir.join(pack_dir.join("maps")))?;
        Ok(())
    }
}
