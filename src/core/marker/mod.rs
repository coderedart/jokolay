use color_eyre::eyre::WrapErr;
use jmf::json::{Marker, Pack};
use jmf::xmlpack::load::ErrorWithLocation;
use jokolink::mlink::Mount;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

// use std::collections::HashMap;
//
// // use crate::json::{pack::ActivationData, SinglePack};
//
// use crate::json::FullPack;
//
// pub struct MarkerManager {}
//
// // #[derive(Debug, Clone, Serialize, Deserialize, Default)]
// // #[serde(default)]
// // pub struct MarkerConfig {}

pub struct MarkerManager {
    pub path: PathBuf,
    pub packs: HashMap<u16, LivePack>,
    pub latest_errors: Option<(Vec<ErrorWithLocation>, Vec<ErrorWithLocation>)>,
}
impl MarkerManager {
    #[tracing::instrument]
    pub async fn new(marker_dir: PathBuf) -> color_eyre::Result<Self> {
        if !marker_dir.exists() {
            tokio::fs::create_dir_all(marker_dir.as_path())
                .await
                .wrap_err("failed to craete marker_packs_dir")?;
        }
        let mut pack_entries = tokio::fs::read_dir(&marker_dir)
            .await
            .wrap_err("failed to read markers directory")?;
        let mut mm = Self {
            path: marker_dir,
            packs: HashMap::new(),
            latest_errors: None,
        };
        while let Some(entry) = pack_entries
            .next_entry()
            .await
            .wrap_err("failed to read next entry of markers dir")?
        {
            if entry
                .file_type()
                .await
                .wrap_err("failed to get entry type while reading markers dir")?
                .is_dir()
            {
                mm.load_pack(entry.path().to_path_buf())
                    .await
                    .wrap_err_with(|| {
                        format!(
                            "failed to load pack from markers dir. pack_dir: {}",
                            entry.path().display()
                        )
                    })?;
            }
        }
        Ok(mm)
    }
    #[tracing::instrument(skip(self))]
    pub async fn load_pack(&mut self, pack_dir: PathBuf) -> color_eyre::Result<LivePack> {
        let pack = Pack::open(pack_dir.join("pack").as_path())
            .await
            .wrap_err_with(|| {
                format!(
                    "failed to load Pack from directory: {}",
                    pack_dir.join("pack").display()
                )
            })?;
        let mut s = String::new();

        tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(pack_dir.join("metadata.json"))
            .await
            .wrap_err("failed to open metadata file of pack")?
            .read_to_string(&mut s)
            .await
            .unwrap_or_default();
        let metadata = serde_json::from_str(&s).unwrap_or_default();
        let lp = LivePack {
            path: pack_dir,
            pack,
            metadata,
            map_cats: Default::default(),
            mount: Default::default(),
            spec: 0,
            live_markers: vec![],
            bind_range: vec![],
        };

        Ok(lp)
    }
    pub async fn import_xml_pack(&mut self) -> color_eyre::Result<()> {
        if let Some(taco_pack_path) = rfd::AsyncFileDialog::new()
            .add_filter("taco", &["taco"])
            .pick_file()
            .await
        {
            let folder = jmf::internal::zpack::extract_zip_to_temp(taco_pack_path.path()).await?;

            let (mut full_pack, errors, warnings) =
                jmf::xmlpack::load::xml_to_json_pack(folder.path());

            self.latest_errors = Some((errors, warnings));
            for i in 0..u16::MAX {
                if !self.packs.contains_key(&i) {
                    let pack_path = self.path.join(format!("{i}/pack"));
                    if pack_path.exists() {
                        tokio::fs::remove_dir_all(&pack_path).await?;
                    }
                    tokio::fs::create_dir_all(&pack_path).await?;
                    full_pack.save_to_folder_multiple_files(&pack_path, true)?;
                    self.load_pack(pack_path).await?;
                    break;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LivePack {
    pub path: PathBuf,
    pub pack: Pack,
    pub metadata: PackMetaData,
    pub map_cats: BTreeSet<u16>,
    pub mount: Mount,
    pub spec: u16,
    pub live_markers: Vec<(u32, Marker)>,
    pub bind_range: Vec<(u16, usize)>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PackMetaData {
    pub activation_data: ActivationData,
    pub enabled_cats: BTreeSet<u16>,
}
/*
visibility:
filters
triggers

on map mesh size:
position
map_display_size,
map_scale
zoom level

in game mesh size:
position of marker
width height from img description
scale
min and max sizes to clamp
distance to the marker itself to lerp between minsize and maxsize
 */
pub struct Mesh {
    pub width: u32,
    pub height: u32,
    pub position: u32,
    pub scale: f32,
    pub color: [u8; 4],
    pub map_display_size: u16,
    pub min_size: u16,
    pub max_size: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ActivationData {
    pub date: time::Date,
    pub account_data: BTreeMap<String, AccountData>,
    pub character_data: BTreeMap<String, CharacterData>,
}
impl Default for ActivationData {
    fn default() -> Self {
        Self {
            date: time::OffsetDateTime::now_utc().date(),
            account_data: Default::default(),
            character_data: Default::default(),
        }
    }
}
/*
pack (edited/deleted) -> current_map (map change) -> enabled cats (cats enabled / edited / deleted / player changed)
-> festival (date_change)
-> mounts (mumble mount change) -> professions (player changed) -> races (player changed) -> specializations ( mumble spec changed)
-> achievement (api update change / player change) -> behavior (activation data changed / reset / player change / instance change / map change)
-> active marker (calculate meshes + check for triggers like tip or info or copy )

 */

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct AccountData {
    pub permanent: BTreeSet<u32>,
    pub daily_reset: BTreeSet<u32>,
    pub timer_based: BTreeMap<u32, time::OffsetDateTime>,
    #[serde(skip)]
    pub instance: HashMap<u32, HashSet<u32>>,
    pub enabled_cats: BTreeSet<u16>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct CharacterData {
    pub daily_reset: BTreeSet<u32>,
    #[serde(skip)]
    pub instance: HashMap<u32, HashSet<u32>>,
}
