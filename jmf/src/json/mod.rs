pub mod category;
pub mod marker;
pub mod trail;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use serde_with::serde_as;
use url::Url;
use validator::Validate;

use crate::{
    json::category::CatTree,
    json::{
        category::Cat,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
pub struct PackDescription {
    // #[validate(length(min = 1))]
    pub name: String,
    pub id: u16,
    pub url: Option<Url>,
    pub git: Option<Url>,
    pub authors: Vec<Author>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
    pub ign: Option<String>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageDescription {
    pub name: String,
    pub width: u16,
    pub height: u16,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackData {
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub images: BTreeMap<u16, Vec<u8>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tbins: BTreeMap<u16, Vec<[f32; 3]>>,
}

#[cfg(test)]
mod tests {
    use crate::json::{Author, ImageDescription};
    use serde_test::*;

    #[test]
    fn serde_author() {
        let author = Author {
            name: "me".to_string(),
            email: Some("me@jokolay.com".to_string()),
            ign: None,
        };

        assert_tokens(
            &author,
            &[
                Token::Struct {
                    name: "Author",
                    len: 2,
                },
                Token::Str("name"),
                Token::String("me"),
                Token::Str("email"),
                Token::Some,
                Token::String("me@jokolay.com"),
                Token::StructEnd,
            ],
        );
    }
    #[test]
    fn serde_image_descriptions() {
        let idesc = ImageDescription {
            name: "waypoint".to_string(),
            width: 128,
            height: 128,
        };

        assert_tokens(
            &idesc,
            &[
                Token::Struct {
                    name: "ImageDescription",
                    len: 3,
                },
                Token::Str("name"),
                Token::String("waypoint"),
                Token::Str("width"),
                Token::U16(128),
                Token::Str("height"),
                Token::U16(128),
                Token::StructEnd,
            ],
        );
    }
}
