use crate::json::{FullPack, Marker};
use jokolink::mlink::Mount;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

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
pub struct MarkerManager {}
pub struct LivePack {
    pub pack: FullPack,
    pub activation_data: ActivationData,
    pub current_map: u16,
    pub map_cats: BTreeSet<u16>,
    pub mount: Mount,
    pub spec: u16,
    pub live_markers: Vec<(u32, Mesh)>,
}
pub struct LiveMarker {
    pub id: u32,
    pub mesh: Mesh,
    pub marker: Marker,
}
pub struct Mesh {}
#[derive(Clone, Serialize, Deserialize)]
pub struct ActivationData {
    pub date: time::Date,
    pub account_data: BTreeMap<String, AccountData>,
    pub character_data: BTreeMap<String, CharacterData>,
}
/*
pack (edited/deleted) -> current_map (map change) -> enabled cats (cats enabled / edited / deleted / player changed)
-> festival (date_change)
-> mounts (mumble mount change) -> professions (player changed) -> races (player changed) -> specializations ( mumble spec changed)
-> achievement (api update change / player change) -> behavior (activation data changed / reset / player change / instance change / map change)
-> active marker (calculate meshes + check for triggers like tip or info or copy )

 */

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AccountData {
    pub permanent: BTreeSet<u32>,
    pub daily_reset: BTreeSet<u32>,
    pub timer_based: BTreeMap<u32, time::OffsetDateTime>,
    #[serde(skip)]
    pub instance: HashMap<u32, HashSet<u32>>,
    pub enabled_cats: BTreeSet<u16>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CharacterData {
    pub daily_reset: BTreeSet<u32>,
    #[serde(skip)]
    pub instance: HashMap<u32, HashSet<u32>>,
}
