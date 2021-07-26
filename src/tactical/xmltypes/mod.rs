use std::{
    ffi::OsStr,
    fs::read_dir,
    sync::{atomic::AtomicBool, Arc},
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

#[derive(Debug)]
pub struct MarCat {
    pub xml_cat: XMLCategory,
    pub markers: Vec<XMLMarker>,
    pub trails: Vec<XMLTrail>,
    pub children: Vec<MarCat>,
    pub enabled: bool,
    pub show_children: bool,
    pub id: u32, //to have a unique id for all the categories to be displayed with egui
}

impl MarCat {
    fn from_od(od: OverlayData, id: &mut u32) -> Option<Self> {
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
        let mut cat = od.categories?;
        let prefix = cat.name.clone();

        let present_cat_markers = markers
            .iter()
            .filter(|&m| {
                if let Some(m_cat) = &m.category {
                    m_cat == &prefix
                } else {
                    true
                }
            })
            .map(|m| m.clone())
            .collect();
        let present_cat_trails = trails
            .iter()
            .filter(|&t| {
                if let Some(t_cat) = &t.category {
                    t_cat == &prefix
                } else {
                    true
                }
            })
            .map(|m| m.clone())
            .collect();

        let mut children = Vec::new();
        if let Some(xml_children) = cat.children {
            children = MarCat::build_mar_cats(prefix, xml_children, &markers, &trails, id);
        }
        cat.children = Some(Vec::new());
        Some(MarCat {
            xml_cat: cat,
            markers: present_cat_markers,
            trails: present_cat_trails,
            children,
            enabled: false,
            id: *id,
            show_children: false,
        })
    }
}
impl MarCat {
    pub fn build_mar_cats(
        prefix: String,
        cats: Vec<XMLCategory>,
        markers: &Vec<XMLMarker>,
        trails: &Vec<XMLTrail>,
        id: &mut u32,
    ) -> Vec<MarCat> {
        let mut result: Vec<MarCat> = Vec::new();

        for mut c in cats {
            let prefix: String = prefix.clone() + "." + &c.name;
            let mut present_cat_markers = markers
                .iter()
                .filter(|&m| {
                    if let Some(m_cat) = &m.category {
                        m_cat == &prefix
                    } else {
                        true
                    }
                })
                .map(|m| m.clone())
                .collect();
            let mut present_cat_trails = trails
                .iter()
                .filter(|&t| {
                    if let Some(t_cat) = &t.category {
                        t_cat == &prefix
                    } else {
                        true
                    }
                })
                .map(|m| m.clone())
                .collect();

            let mut children = Vec::new();
            if let Some(xml_children) = c.children {
                children = MarCat::build_mar_cats(prefix, xml_children, &markers, &trails, id);
            }
            c.children = Some(Vec::new());
            if let Some(v) = result.iter_mut().find(|m| m.xml_cat.name == c.name) {
                v.markers.append(&mut present_cat_markers);
                v.trails.append(&mut present_cat_trails);
                v.children.append(&mut children);
            } else {
                result.push(MarCat {
                    xml_cat: c,
                    markers: present_cat_markers,
                    trails: present_cat_trails,
                    children,
                    enabled: false,
                    id: *id,
                    show_children: false,
                });
                *id += 1;
            }
        }
        return result;
    }
}
pub fn merge(original: &mut Vec<MarCat>, other: Vec<MarCat>) {
    for mut v in other {
        if let Some(ori_v) = original
            .iter_mut()
            .find(|c| c.xml_cat.name == v.xml_cat.name)
        {
            ori_v.markers.append(&mut v.markers);
            ori_v.trails.append(&mut v.trails);
            merge(&mut ori_v.children, v.children);
        } else {
            original.push(v);
        }
    }
}
pub fn load_markers(location: &str) -> anyhow::Result<Vec<MarCat>> {
    let mut mar_cats: Vec<MarCat> = Vec::new();
    let mut id = 0u32;
    let entries = read_dir(&location).context(format!("couldn't open directory {}", &location))?;
    for f in entries {
        let entry = f?;

        if entry.path().extension() == Some(OsStr::new("xml")) {
            let marker_file = std::fs::File::open(&entry.path()).unwrap();
            let marker_file_reader = std::io::BufReader::new(marker_file);

            match quick_xml::de::from_reader::<_, OverlayData>(marker_file_reader) {
                Ok(od) => {
                    let mut new_cat_map = Vec::new();
                    if let Some(new_cat) = MarCat::from_od(od, &mut id) {
                        new_cat_map.push(new_cat);
                        merge(&mut mar_cats, new_cat_map);
                    } else {
                        log::warn!(
                            "failed to get Marcat from file {:?} due to no MarkerCategory tag",
                            &entry.path()
                        );
                    }
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

pub struct CatDisplay {
    pub template: XMLCategory,
    pub enabled: Arc<AtomicBool>,
    pub children: Vec<CatDisplay>,
}
