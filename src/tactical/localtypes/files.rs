use crate::{
    core::fm::{FileManager, RID},
    tactical::{
        localtypes::{
            category::{CatSelectionTree, IMCategory, MCIndexTree},
            marker::MarkerTemplate,
        },
        xmltypes::{xml_category::OverlayData, xml_marker::XMLPOI, xml_trail::XMLTrail},
    },
};
use quick_xml::de::Deserializer;
// use quick_xml::de::from_reader as xmlreader;
// use serde_xml_rs::de::from_reader as xmlreader;
use std::{collections::HashMap, io::BufReader};
use uuid::Uuid;
use vfs::VfsPath;

/// Marker File is the primary abstraction to use for editing markerpacks. they can only have one Active Marker File to edit.
/// it is a abstract representation of the `OverlayData` struct which we deserialize from xml files. this helps us keep all the markers/cats in one place
/// while also have this struct represent the OverlayData struct with just their Uuids/indexes. when we want to create/edit an existing marker file, this is
/// the primary struct to use as it will remember the path of the file and overwrite it with the changes when necessary.
#[derive(Debug, Clone)]
pub struct MarkerFile {
    pub path: RID,
    pub mc_index_tree: Option<MCIndexTree>,
    pub poi_vec: Vec<Uuid>,
    pub trl_vec: Vec<Uuid>,
}

impl MarkerFile {
    pub fn parse_marker_file(
        pack_path: RID,
        fm: &FileManager,
        file_path: VfsPath,
        global_cats: &mut Vec<IMCategory>,
        global_pois: &mut HashMap<Uuid, XMLPOI>,
        global_trails: &mut HashMap<Uuid, XMLTrail>,
        name_id_map: &mut HashMap<String, usize>,
        mfiles: &mut Vec<MarkerFile>,
        cstree: &mut Vec<CatSelectionTree>,
    ) {
        let xml_file = file_path
            .open_file()
            .map_err(|e| {
                log::error!(
                    "couldn't open xml file: {:?} due to error: {:?}",
                    file_path.as_str(),
                    &e
                );
                e
            })
            .unwrap();
        let marker_file_reader = BufReader::new(xml_file);
        let de = &mut Deserializer::from_reader(marker_file_reader);
        let od: OverlayData = match serde_path_to_error::deserialize(de) {
            Ok(od) => od,
            Err(e) => {
                log::error!(
                    "failed to deserialize file {:?} at {} due to error: {}\n",
                    file_path.as_str(),
                    e.path().to_string(),
                    e.into_inner()
                );
                return;
            }
        };

        let mut mc_index_tree = Vec::new();
        if let Some(mctree) = od.categories {
            MCIndexTree::index_tree_from_mc_tree(
                pack_path,
                fm,
                vec![mctree],
                &mut mc_index_tree,
                global_cats,
                MarkerTemplate::default(),
                "",
                name_id_map,
            )
        }
        let mut uuid_poi_vec = Vec::new();
        let mut uuid_trail_vec = Vec::new();

        if let Some(pois) = od.pois {
            // seperate poi and trail tags
            let mut poi_vec: Option<Vec<XMLPOI>> = None;
            let mut trail_vec: Option<Vec<XMLTrail>> = None;
            if let Some(tags) = pois.tags {
                for tag in tags {
                    match tag {
                        crate::tactical::xmltypes::xml_marker::PoiOrTrail::POI(p) => {
                            match poi_vec {
                                Some(ref mut v) => v.push(p),
                                None => poi_vec = Some(vec![p]),
                            }
                        }
                        crate::tactical::xmltypes::xml_marker::PoiOrTrail::Trail(t) => {
                            match trail_vec {
                                Some(ref mut v) => v.push(t),
                                None => trail_vec = Some(vec![t]),
                            }
                        }
                        crate::tactical::xmltypes::xml_marker::PoiOrTrail::Route(_) => {}
                    }
                }
            }
            if let Some(vp) = poi_vec {
                uuid_poi_vec = vp
                    .into_iter()
                    .map(|mut p| {
                        let id = p.guid.unwrap_or(Uuid::new_v4());
                        p.guid = Some(id);
                        if let Some(_) = global_pois.insert(id, p) {
                            log::error!("two markers have the same guid: {} ", &id);
                        }
                        id
                    })
                    .collect();
            }
            if let Some(vt) = trail_vec {
                uuid_trail_vec = vt
                    .into_iter()
                    .map(|mut t| {
                        let id = t.guid.unwrap_or(Uuid::new_v4());
                        t.guid = Some(id);
                        if let Some(_) = global_trails.insert(id, t) {
                            log::error!("two trails have the same guid: {} ", &id);
                        }
                        id
                    })
                    .collect();
            }
        }
        CatSelectionTree::build_cat_selection_tree(&mc_index_tree, cstree);
        let mc_index_tree = if !mc_index_tree.is_empty() {
            Some(mc_index_tree.remove(0))
        } else {
            None
        };
        let vid = fm.get_vid(&file_path).unwrap();
        mfiles.push(MarkerFile {
            path: vid,
            mc_index_tree,
            poi_vec: uuid_poi_vec,
            trl_vec: uuid_trail_vec,
        });
    }
}
