use crate::json::author::Author;
use crate::json::category::{Cat, CatTree};
use crate::json::marker::Marker;
use crate::json::trail::{TBinDescription, Trail};
use crate::json::{Dirty, ImageDescription};
use color_eyre::eyre::{ContextCompat, WrapErr};
use serde::*;
use serde_with::*;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::instrument;
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FullPack {
    pub pack: Pack,
    pub pack_data: PackData,
    /// tells us whether this has been changed since loading i.e. whether this needs to be saved to file
    #[serde(skip)]
    pub dirty: Dirty,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Pack {
    pub pack_description: PackDescription,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub images_descriptions: BTreeMap<u16, ImageDescription>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tbins_descriptions: BTreeMap<u16, TBinDescription>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub strings: BTreeMap<u16, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub cats: BTreeMap<u16, Cat>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cat_tree: Vec<CatTree>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub markers: BTreeMap<u32, Marker>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub trails: BTreeMap<u32, Trail>,
}

/// This contains all the images and Tbin files referred to by their ID
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
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
    #[instrument]
    pub async fn open(pack_dir: &Path) -> color_eyre::Result<Self> {
        let mut buffer = String::new();
        File::open(&pack_dir.join("pack_description.json"))
            .await?
            .read_to_string(&mut buffer)
            .await?;
        let pack_description =
            serde_json::from_str(&buffer).wrap_err("failed to deserialize pack_desc.json")?;
        buffer.clear();
        File::open(&pack_dir.join("images_descriptions.json"))
            .await?
            .read_to_string(&mut buffer)
            .await?;
        let images_descriptions = serde_json::from_str(&buffer)
            .wrap_err("failed to deserialize images_description.json")?;
        buffer.clear();
        File::open(&pack_dir.join("tbins_descriptions.json"))
            .await?
            .read_to_string(&mut buffer)
            .await?;
        let tbins_descriptions = serde_json::from_str(&buffer)
            .wrap_err("failed to deserialize tbins_description.json")?;
        buffer.clear();
        File::open(&pack_dir.join("strings.json"))
            .await?
            .read_to_string(&mut buffer)
            .await?;
        let strings =
            serde_json::from_str(&buffer).wrap_err("failed to deserialize strings.json")?;
        buffer.clear();
        File::open(&pack_dir.join("cats_descriptions.json"))
            .await?
            .read_to_string(&mut buffer)
            .await?;
        let cats = serde_json::from_str(&buffer).wrap_err("failed to deserialize cats.json")?;
        buffer.clear();
        File::open(&pack_dir.join("cat_tree.json"))
            .await?
            .read_to_string(&mut buffer)
            .await?;
        let cat_tree =
            serde_json::from_str(&buffer).wrap_err("failed to deserialize cat_tree.json")?;
        buffer.clear();
        let mut markers: BTreeMap<u32, Marker> = Default::default();
        let mut entries = fs::read_dir(&pack_dir.join("markers")).await?;
        while let Some(marker_file_entry) = entries.next_entry().await? {
            buffer.clear();
            File::open(marker_file_entry.path())
                .await?
                .read_to_string(&mut buffer)
                .await?;
            let map_markers: BTreeMap<u32, Marker> =
                serde_json::from_str(&buffer).wrap_err_with(|| {
                    format!(
                        "failed to deserialize {}",
                        marker_file_entry.path().display()
                    )
                })?;
            markers.extend(map_markers);
        }
        let mut trails: BTreeMap<u32, Trail> = Default::default();
        let mut entries = fs::read_dir(&pack_dir.join("trails")).await?;
        while let Some(trail_file_entry) = entries.next_entry().await? {
            buffer.clear();
            File::open(trail_file_entry.path())
                .await?
                .read_to_string(&mut buffer)
                .await?;
            let map_trails: BTreeMap<u32, Trail> =
                serde_json::from_str(&buffer).wrap_err_with(|| {
                    format!(
                        "failed to deserialize {}",
                        trail_file_entry.path().display()
                    )
                })?;
            trails.extend(map_trails);
        }
        Ok(Self {
            pack_description,
            images_descriptions,
            tbins_descriptions,
            strings,
            cats,
            cat_tree,
            markers,
            trails,
        })
    }
    #[tracing::instrument(skip(self))]
    pub fn save_to_folder_multiple_files(
        &self,
        pack_dir: &Path,
        dirty: &Dirty,
    ) -> color_eyre::Result<()> {
        if dirty.pack_desc {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(
                    &pack_dir.join("pack_description.json"),
                )?),
                &self.pack_description,
            )?;
        }
        if dirty.image_desc {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(
                    &pack_dir.join("images_descriptions.json"),
                )?),
                &self.images_descriptions,
            )?;
        }
        if dirty.tbin_desc {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(
                    &pack_dir.join("tbins_descriptions.json"),
                )?),
                &self.tbins_descriptions,
            )?;
        }
        if dirty.string_desc {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(&pack_dir.join("strings.json"))?),
                &self.strings,
            )?;
        }
        if dirty.cat_desc {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(
                    &pack_dir.join("cats_descriptions.json"),
                )?),
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
            let end_marker_id = start_marker_id + u16::MAX as u32;
            let present_map_markers: BTreeMap<_, _> = self
                .markers
                .range(start_marker_id..=end_marker_id)
                .collect();
            let map_markers_path = markers_dir.join(&format!("{map_id}.json"));
            if present_map_markers.is_empty() {
                if map_markers_path.exists() {
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
            let end_trail_id = start_trail_id + u16::MAX as u32;
            let present_map_trails: BTreeMap<_, _> =
                self.trails.range(start_trail_id..=end_trail_id).collect();
            let map_trails_path = trails_dir.join(&format!("{map_id}.json"));
            if present_map_trails.is_empty() {
                if map_trails_path.exists() {
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
    pub async fn open(pack_dir: &Path) -> color_eyre::Result<Self> {
        let mut images: BTreeMap<u16, Vec<u8>> = Default::default();
        let mut entries = fs::read_dir(&pack_dir.join("images")).await?;
        while let Some(entry) = entries.next_entry().await? {
            let key = entry
                .path()
                .file_stem()
                .wrap_err("no file stem")?
                .to_str()
                .unwrap()
                .parse()?;

            let mut buffer = vec![];
            File::open(entry.path())
                .await?
                .read_to_end(&mut buffer)
                .await?;

            images.insert(key, buffer);
        }
        let mut tbins: BTreeMap<u16, Vec<[f32; 3]>> = Default::default();
        let mut entries = fs::read_dir(&pack_dir.join("tbins")).await?;
        while let Some(entry) = entries.next_entry().await? {
            let key = entry
                .path()
                .file_stem()
                .wrap_err("failed to get file stem")?
                .to_str()
                .unwrap()
                .parse()?;
            let mut buffer = vec![];
            File::open(entry.path())
                .await?
                .read_to_end(&mut buffer)
                .await?;
            let buffer: Vec<[f32; 3]> = bytemuck::cast_slice(buffer.as_slice()).to_vec();
            tbins.insert(key, buffer);
        }

        Ok(Self { images, tbins })
    }
    pub fn save_to_folder_multiple_files(
        &self,
        pack_dir: &Path,
        dirty: &Dirty,
    ) -> color_eyre::Result<()> {
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
                std::fs::File::create(&tbin_path)?
                    .write_all(bytemuck::cast_slice(tbin_bytes.as_slice()))?;
            } else if tbin_path.exists() {
                std::fs::remove_file(&tbin_path)?;
            }
        }
        Ok(())
    }
}

impl FullPack {
    pub async fn open(pack_dir: &Path) -> color_eyre::Result<Self> {
        let pack = Pack::open(pack_dir).await?;
        let pack_data = PackData::open(pack_dir).await?;

        Ok(Self {
            pack,
            pack_data,
            dirty: Default::default(),
        })
    }
    pub fn save_to_folder_multiple_files(
        &mut self,
        pack_dir: &Path,
        full_save: bool,
    ) -> color_eyre::Result<()> {
        if full_save {
            self.dirty = Dirty::full_from_pack(self);
        }
        self.pack
            .save_to_folder_multiple_files(pack_dir, &self.dirty)?;
        self.pack_data
            .save_to_folder_multiple_files(pack_dir, &self.dirty)?;
        self.dirty = Dirty::default();
        Ok(())
    }
}
