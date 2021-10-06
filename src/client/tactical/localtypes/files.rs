use crate::client::{am::AssetManager, tactical::{localtypes::{category::{CatSelectionTree, IMCategory, MCIndexTree}, marker::{MarkerTemplate, POI}, trail::Trail}, xmltypes::{xml_category::OverlayData, xml_marker::{PoiOrTrail}}}};
use anyhow::Context;
use quick_xml::de::Deserializer;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use std::collections::HashMap;
use uuid::Uuid;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Default, Deserialize,
)]
pub struct MarkerFileIndex(pub usize);
/// Marker File is the primary abstraction to use for editing markerpacks. they can only have one Active Marker File to edit.
/// it is a abstract representation of the `OverlayData` struct which we deserialize from xml files. this helps us keep all the markers/cats in one place
/// while also have this struct represent the OverlayData struct with just their Uuids/indexes. when we want to create/edit an existing marker file, this is
/// the primary struct to use as it will remember the path of the file and overwrite it with the changes when necessary.
#[derive(Debug, Clone)]
pub struct MarkerFile {
    pub path: usize,
    pub mc_index_tree: Vec<MCIndexTree>,
    pub poi_vec: Vec<usize>,
    pub trl_vec: Vec<usize>,
}

impl MarkerFile {
    pub async fn new(
        pack_path_id: usize,
        mfile_id: usize,
        am: &mut AssetManager,
        global_cats: &mut Vec<IMCategory>,
        global_pois: &mut Vec<POI>,
        global_trails: &mut Vec<Trail>,
        name_id_map: &mut HashMap<String, usize>,
        cstree: &mut Vec<CatSelectionTree>,
    ) -> anyhow::Result<MarkerFile> {
        let mut xml_file_contents = vec![];
        let mut file = am.open_file(mfile_id).await?;
        file.read_to_end(&mut xml_file_contents);
        let xml_string = String::from_utf8(xml_file_contents)?;
        let reader = quick_xml::Reader::from_str(&xml_string);
        let de = &mut Deserializer::new(reader);
        let od: OverlayData = serde_path_to_error::deserialize(de).context(format!(
            "failed to deserialize file {:?} ",
            am.get_file_path_from_id(mfile_id)
        ))?;

        let mut mc_index_tree = Vec::new();
        if let Some(raw_xmlmc_tree) = od.categories {
            MCIndexTree::index_tree_from_mc_tree(
                pack_path_id,
                am,
                vec![raw_xmlmc_tree],
                &mut mc_index_tree,
                global_cats,
                MarkerTemplate::default(),
                "",
                name_id_map,
            )?
        }
        let mut poi_vec = Vec::new();
        let mut trl_vec = Vec::new();

        if let Some(pois) = od.pois {
            if let Some(tags) = pois.tags {
                for tag in tags {
                    match tag {
                        PoiOrTrail::POI(mut p) => {
                            if p.guid.is_none() {
                                p.guid = Some(Uuid::new_v4());
                            } 
                            let p = POI::from_xmlpoi(pack_path_id, &p, global_cats, am)?;
                            let index = global_pois.len();
                            global_pois.push(p);
                            poi_vec.push(index);
                        }
                        PoiOrTrail::Trail(mut t) => {
                            if t.guid.is_none() {
                                t.guid = Some(Uuid::new_v4());
                            }
                            let t = Trail::from_xml_trail(pack_path_id, &t, global_cats, am).await?;
                            let index = global_trails.len();
                            global_trails.push(t);
                            trl_vec.push(index);

                        }
                        PoiOrTrail::Route(_) => {}
                    }
                }
            }
     
        }
        CatSelectionTree::build_cat_selection_tree(&mc_index_tree, cstree);

     
        Ok(MarkerFile {
            path: mfile_id,
            mc_index_tree,
            poi_vec,
            trl_vec,
        })
    }
}
