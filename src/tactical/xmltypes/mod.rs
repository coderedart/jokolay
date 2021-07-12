use std::{
    ffi::OsStr,
    fs::read_dir,
};

use anyhow::Context;

use crate::tactical::xmltypes::xml_category::OverlayData;

use {
    xml_category::MarkerCategory as XMLCategory, xml_marker::Marker as XMLMarker,
    xml_trail::Trail as XMLTrail,
};

pub mod xml_category;
pub mod xml_marker;
pub mod xml_trail;

pub struct MarCat {
    pub xml_cat: XMLCategory,
    pub markers: Vec<XMLMarker>,
    pub trails: Vec<XMLTrail>,
    pub children: Vec<MarCat>,
    pub enabled: bool,
}

impl From<OverlayData> for MarCat {
    fn from(od: OverlayData) -> Self {
        let mut markers: Vec<XMLMarker> = Vec::new();
        let mut trails: Vec<XMLTrail> = Vec::new();

        if let Some(pois) = od.pois {
            if let Some(m) = pois.poi {
                markers = m;
            }
            if let Some(t) = pois.trail {
                trails = t;
            }
        }
        let mut cat = od.categories;
        let prefix = cat.name.clone();

        
        let present_cat_markers = markers.iter().filter(|&m| {
            if let Some(m_cat) = &m.category {
                m_cat == &prefix
            } else {
                true
            }
        }).map(|m| m.clone()).collect();
        let present_cat_trails = trails.iter().filter(|&t| {
            if let Some(t_cat) = &t.category {
                t_cat == &prefix
            } else {
                true
            }
        }).map(|m|m.clone()).collect();

        let mut children = Vec::new();
        if let Some(xml_children) = cat.children {
            children = MarCat::build_mar_cats(prefix, xml_children, &markers, &trails);
        }
        cat.children = Some(Vec::new());
        MarCat {
            xml_cat: cat,
            markers: present_cat_markers,
            trails: present_cat_trails,
            children,
            enabled: false,
        }
    }
}
impl MarCat {
    pub fn build_mar_cats(
        prefix: String,
        cats: Vec<XMLCategory>,
         markers: &Vec<XMLMarker>,
         trails: &Vec<XMLTrail>,
    ) -> Vec<MarCat> {
        let mut result = Vec::new();

        for mut c in cats {
            let prefix: String  = prefix.clone() + "." + &c.name;
            let present_cat_markers = markers.iter().filter(|&m| {
                if let Some(m_cat) = &m.category {
                    m_cat == &prefix
                } else {
                    true
                }
            }).map(|m| m.clone()).collect();
            let present_cat_trails = trails.iter().filter(|&t| {
                if let Some(t_cat) = &t.category {
                    t_cat == &prefix
                } else {
                    true
                }
            }).map(|m|m.clone()).collect();

            let mut children = Vec::new();
            if let Some(xml_children) = c.children {
                children = MarCat::build_mar_cats(prefix, xml_children, &markers, &trails);
            }
            c.children = Some(Vec::new());
            result.push(MarCat {
                xml_cat: c,
                markers: present_cat_markers,
                trails: present_cat_trails,
                children,
                enabled: false,
            });
        }
        return result;
    }
}

pub fn load_markers(location: &str) -> anyhow::Result<Vec<MarCat>> {
    let mut mar_cats = Vec::new();
    let entries = read_dir(&location).context(format!("couldn't open directory {}", &location))?;
    for f in entries {
        let entry = f?;

        if entry.path().extension() == Some(OsStr::new("xml")) {
            let marker_file = std::fs::File::open(&entry.path()).unwrap();
            let marker_file_reader = std::io::BufReader::new(marker_file);

            match quick_xml::de::from_reader::<_, OverlayData>(marker_file_reader) {
                Ok(od) => {
                    mar_cats.push(MarCat::from(od));
                }
                Err(e) => {
                    log::error!(
                        "failed to deserialize file {:?} due to error {}\n",
                        entry.path(),
                        e
                    )
                }
            }
        }
    }
    // let mut trail_map = BTreeMap::new();
    // for t in trails.into_iter() {
    //     let trail_path = t.trail_data.as_ref().unwrap();
    //     let trail_file = std::fs::File::open(format!("{}/{}", "./res/tw", trail_path)).unwrap();
    //     let mut trail_reader = std::io::BufReader::new(trail_file);
    //     let mut buffer_u32 = [0_u8; 4];
    //     trail_reader.read(&mut buffer_u32).unwrap();
    //     let map_id = u32::from_ne_bytes(buffer_u32);
    //     trail_map.entry(map_id).or_insert(Vec::new()).push(t);
    // }
    Ok(mar_cats)
}
