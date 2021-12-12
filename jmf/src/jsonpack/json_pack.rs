use std::hash::Hash;

use serde_with::serde_as;

use jokotypes::*;
use serde::{Deserialize, Serialize, Serializer};
use url::Url;

use crate::{
    jsonpack::json_cat::{CatSelectionTree, JsonCat},
    jsonpack::json_trail::TBinDescription,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct PackDescription {
    pub name: String,
    pub id: PackID,
    pub url: Option<Url>,
    pub git: Option<Url>,
    pub authors: Vec<Author>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Default, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
    pub ign: Option<String>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageDescription {
    pub name: String,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackStatus {
    pub activation_data: ActivationData,
    pub next_check: UTStamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ActivationData {
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub activation_times: UOMap<MarkerID, UTStamp>,
    #[serde(default)]
    #[serde(skip_serializing_if = "UOSet::is_empty")]
    pub activated_cats: UOSet<CategoryID>,
}

impl ActivationData {
    pub fn is_empty(&self) -> bool {
        self.activated_cats.is_empty() && self.activation_times.is_empty()
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonPack {
    pub pack_description: PackDescription,
    #[serde(serialize_with = "ordered_map")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub images_descriptions: UOMap<ImageHash, ImageDescription>,
    #[serde(serialize_with = "ordered_map")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub tbins_descriptions: UOMap<TrailHash, TBinDescription>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cattree: Vec<CatSelectionTree>,
    #[serde(serialize_with = "ordered_map")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub cats: UOMap<CategoryID, JsonCat>,
}
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackData {
    #[serde(with = "map_base64")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub images: UOMap<ImageHash, Vec<u8>>,
    #[serde(with = "map_base64")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub tbins: UOMap<TrailHash, Vec<[f32; 3]>>,
}

pub fn ordered_map<K, V, S>(value: &UOMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Ord + Serialize + Hash + Clone,
    V: Serialize + Clone,
    S: Serializer,
{
    let ordered: OMap<K, V> = value.clone().into();
    ordered.serialize(serializer)
}

pub mod map_base64 {

    use bytemuck::Pod;
    use jokotypes::{OMap, UOMap};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<K, V, S>(value: &UOMap<K, Vec<V>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: Ord + Serialize + Clone + std::hash::Hash,
        V: Serialize + Clone + Pod,
        S: Serializer,
    {
        let ordered: OMap<_, _> = value
            .iter()
            .map(|(k, v)| {
                let k = k.clone();
                let v = base64::encode(bytemuck::cast_slice(&v[..]));
                (k, v)
            })
            .collect();
        ordered.serialize(serializer)
    }
    pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<UOMap<K, Vec<V>>, D::Error>
    where
        K: Ord + Deserialize<'de> + Clone + std::hash::Hash,
        V: Deserialize<'de> + Clone + Pod,
        D: Deserializer<'de>,
    {
        let m: std::collections::HashMap<K, String> =
            std::collections::HashMap::deserialize(deserializer)?;
        Ok((m.into_iter().map(|(k, v)| {
            let v = base64::decode(v).unwrap();
            let v: &[V] = bytemuck::cast_slice(&v);
            let v = v.to_vec();
            (k, v)
        }))
        .collect())
    }
}
