
use jmf::json::{Marker, Pack};
use jokolink::mlink::Mount;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{PathBuf};

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
    pub packs: Vec<LivePack>,
}
impl MarkerManager {
    pub async fn new(marker_dir: PathBuf) -> color_eyre::Result<Self> {
        Ok(Self {
            path: marker_dir,
            packs: vec![],
        })
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
