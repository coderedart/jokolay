use crate::{
    fm::{FileManager, VID},
    tactical::{
        localtypes::{
            category::{CatSelectionTree, IMCategory, MCIndexTree},
            marker::{MarkerTemplate, POI},
            trail::Trail,
        },
        xmltypes::xml_category::OverlayData,
    },
};
use quick_xml::de::from_reader as xmlreader;

use std::{collections::HashMap, io::BufReader};
use uuid::Uuid;
use vfs::VfsPath;

/// Marker File is the primary abstraction to use for editing markerpacks. they can only have one Active Marker File to edit.
/// it is a abstract representation of the `OverlayData` struct which we deserialize from xml files. this helps us keep all the markers/cats in one place
/// while also have this struct represent the OverlayData struct with just their Uuids/indexes. when we want to create/edit an existing marker file, this is
/// the primary struct to use as it will remember the path of the file and overwrite it with the changes when necessary.
#[derive(Debug, Clone)]
pub struct MarkerFile {
    pub path: VID,
    pub mc_index_tree: Option<MCIndexTree>,
    pub poi_vec: Vec<Uuid>,
    pub trl_vec: Vec<Uuid>,
    // pub changes: Option<Vec<MarkerEditAction>>,
}

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
// pub enum MarkerEditAction {
//     CreateCategory {
//         location: Vec<usize>,
//         new_category: XMLMarkerCategory,
//     },
//     UpdateCategory {
//         location: Vec<usize>,
//         previous: XMLMarkerCategory,
//         updated: XMLMarkerCategory,
//     },
//     DeleteCategory {
//         location: Vec<usize>,
//         deleted_item: XMLMarkerCategory,
//     },
//     CreateMarker {
//         location: usize,
//         new_marker: XMLPOI,
//     },
//     UpdateMarker {
//         location: usize,
//         previous: XMLPOI,
//         updated: XMLPOI,
//     },
//     DeleteMarker {
//         location: usize,
//         deleted_item: XMLPOI,
//     },
// }

impl MarkerFile {
    pub fn parse_marker_file(
        pack_path: VID,
        fm: &FileManager,
        file_path: VfsPath,
        global_cats: &mut Vec<IMCategory>,
        global_pois: &mut HashMap<Uuid, POI>,
        global_trails: &mut HashMap<Uuid, Trail>,
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

        let od: OverlayData = match xmlreader(marker_file_reader) {
            Ok(od) => od,
            Err(e) => {
                log::error!(
                    "failed to deserialize file {:?} due to error {:?}\n",
                    file_path.as_str(),
                    e
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
        if let Some(p) = od.pois {
            if let Some(vp) = p.poi {
                uuid_poi_vec = POI::get_vec_uuid_pois(vp, global_pois, pack_path, global_cats, fm);
            }
            if let Some(vt) = p.trail {
                uuid_trail_vec =
                    Trail::get_vec_uuid_trail(vt, global_trails, pack_path, global_cats, fm);
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
