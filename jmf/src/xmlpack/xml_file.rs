use crate::xmlpack::{
    xml_category::XMLMarkerCategory,
    xml_marker::{POIs, SerializePOIs},
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use std::path::PathBuf;

/// holds the OverlayData contents and the path of the file.
#[derive(Debug, Clone)]
pub struct XmlFile {
    pub path: PathBuf,
    pub od: OverlayData,
}

/// The root overlay tag in any valid marker xml file. and use with serde directly for easy serialize and deserialize
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OverlayData {
    #[serde(rename = "MarkerCategory")]
    pub categories: Option<Vec<XMLMarkerCategory>>,
    #[serde(rename = "POIs")]
    pub pois: Option<POIs>,
}

/// The root overlay tag in any valid marker xml file. and use with serde directly for easy serialize and deserialize
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SerializeOverlayData {
    #[serde(rename = "MarkerCategory")]
    pub categories: Option<Vec<XMLMarkerCategory>>,
    #[serde(rename = "POIs")]
    pub pois: Option<SerializePOIs>,
}

pub fn deserialize_od(src_xml: &str) -> Result<OverlayData, quick_xml::DeError> {
    
    let mut de = quick_xml::de::Deserializer::from_reader(std::io::Cursor::new(src_xml));
     if let Err(e) = serde_path_to_error::deserialize::<'_, _, OverlayData>(&mut de) {
         dbg!(e);
     }
     quick_xml::de::from_str(src_xml)
}

pub fn serialize_od(od: &OverlayData) -> Result<String, quick_xml::DeError> {
    quick_xml::se::to_string(od)
}