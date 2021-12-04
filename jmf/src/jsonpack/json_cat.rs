use crate::jsonpack::{
    json_marker::Marker, json_pack::ordered_map, json_pack::Author, json_trail::Trail,
};
use jokotypes::*;
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct CatDescription {
    pub name: String,
    pub display_name: String,
    pub id: CategoryID,
    pub is_separator: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct MapMarkers {
    pub map_id: MapID,
    #[serde(serialize_with = "ordered_map")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub markers: UOMap<MarkerID, Marker>,
    #[serde(serialize_with = "ordered_map")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub trails: UOMap<TrailID, Trail>,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct Tags {
    #[serde(serialize_with = "ordered_map")]
    pub tags: UOMap<String, OSet<MarkerID>>,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CatSelectionTree {
    pub name: String,
    pub id: CategoryID,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CatSelectionTree>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonCat {
    pub cat_description: CatDescription,
    pub tags: Tags,
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub map_markers: UOMap<MapID, MapMarkers>,
}
