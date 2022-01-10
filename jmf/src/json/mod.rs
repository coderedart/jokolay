pub mod category;
pub mod marker;
pub mod trail;

use serde::{Deserialize, Serialize};


use jokotypes::*;
use serde_with::serde_as;
use serde_with::FromInto;
use url::Url;
use validator::Validate;

use crate::{
    json::category::CatSelectionTree,
    json::{
        category::CatDescription,
        marker::Marker,
        trail::{TBinDescription, Trail},
    },
};

#[derive(Debug, Clone)]
pub struct FullPack {
    pub pack: Pack,
    pub pack_data: PackData,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PackDescription {
    #[validate(length(min = 1))]
    pub name: String,
    pub id: PackID,
    pub url: Option<Url>,
    pub git: Option<Url>,
    pub authors: Vec<Author>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize,  Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct PackStatus {
//     pub activation_data: ActivationData,
//     pub next_check: UTStamp,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
// #[serde(default)]
// pub struct ActivationData {
//     #[serde(default)]
//     #[serde(skip_serializing_if = "UOMap::is_empty")]
//     pub activation_times: UOMap<MarkerID, UTStamp>,
//     #[serde(default)]
//     #[serde(skip_serializing_if = "UOMap::is_empty")]
//     pub activated_cats: UOMap<CategoryID, bool>,
// }

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub pack_description: PackDescription,
    #[serde_as(as = "FromInto<OMap<_, _>")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub images_descriptions: UOMap<ImageID, ImageDescription>,
    #[serde_as(as = "FromInto<OMap<_, _>")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub tbins_descriptions: UOMap<TrailID, TBinDescription>,
    #[serde_as(as = "FromInto<OMap<_, _>")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub cats: UOMap<CategoryID, CatDescription>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cattree: Vec<CatSelectionTree>,
    #[serde_as(as = "FromInto<OMap<_, _>")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub markers: UOMap<MarkerID, Marker>,
    #[serde_as(as = "FromInto<OMap<_, _>")]
    #[serde(default)]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub trails: UOMap<TrailID, Trail>,
}


#[derive(Debug, Clone, Default)]
pub struct PackData {
    // #[serde(default)]
    // #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub images: UOMap<ImageID, Vec<u8>>,
    // #[serde(default)]
    // #[serde(skip_serializing_if = "UOMap::is_empty")]
    pub tbins: UOMap<TrailID, Vec<[f32; 3]>>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize,  Deserialize)]
#[serde(transparent)]
pub struct Tags(
    #[serde_as(as = "FromInto<OMap<_, _>")]
    #[serde(skip_serializing_if = "UOMap::is_empty")]
    UOMap<String, OSet<MarkerID>>,
);


// pub mod map_base64 {

//     use bytemuck::Pod;
//     use jokotypes::{OMap, UOMap};
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};

//     pub fn serialize<K, V, S>(value: &UOMap<K, Vec<V>>, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         K: Ord + Serialize + Clone + std::hash::Hash,
//         V: Serialize + Clone + Pod,
//         S: Serializer,
//     {
//         let ordered: OMap<_, _> = value
//             .iter()
//             .map(|(k, v)| {
//                 let k = k.clone();
//                 let v = base64::encode(bytemuck::cast_slice(&v[..]));
//                 (k, v)
//             })
//             .collect();
//         ordered.serialize(serializer)
//     }
//     pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<UOMap<K, Vec<V>>, D::Error>
//     where
//         K: Ord + Deserialize<'de> + Clone + std::hash::Hash,
//         V: Deserialize<'de> + Clone + Pod,
//         D: Deserializer<'de>,
//     {
//         let m: std::collections::HashMap<K, String> =
//             std::collections::HashMap::deserialize(deserializer)?;
//         Ok((m.into_iter().map(|(k, v)| {
//             let v = base64::decode(v).unwrap();
//             let v: &[V] = bytemuck::cast_slice(&v);
//             let v = v.to_vec();
//             (k, v)
//         }))
//         .collect())
//     }
// }
