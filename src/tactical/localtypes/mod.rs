pub mod category;
pub mod files;
pub mod manager;
pub mod marker;
pub mod trail;
use crate::{fm::{FileManager, VID}, tactical::{localtypes::{category::{CatSelectionTree, IMCategory, MCIndexTree}, files::{MarkerFile}, marker::{MarkerTemplate, POI}, trail::Trail}, xmltypes::{
            xml_category::{OverlayData, XMLMarkerCategory},
            xml_marker::XMLPOI,
            xml_trail::XMLTrail,
        }}};
use quick_xml::de::from_reader as xmlreader;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs::read_dir,
    io::BufReader,
    path::PathBuf,
};
use uuid::Uuid;
use vfs::VfsPath;

/// Zip Crate is getting a new API overhaul soon. so, until then just use normal forlders. The pack itself should be self-contained including the images/other file references relative to this.
#[derive(Debug, Clone)]
pub struct MarkerPack {
    /// The path to the folder where the marker xml files and other data live.
    pub path: VID,
    /// the marker files collected so that we can later just turn them back into overlaydata if we have changes.
    pub mfiles: Vec<MarkerFile>,
    /// The categories all stored in a Vector and referenced by other markers/trails via the index into this vector
    pub global_cats: Vec<IMCategory>,
    /// all the POIs in the current pack.
    pub global_pois: HashMap<Uuid, POI>,
    /// All the trail `tags` in the current pack.
    pub global_trails: HashMap<Uuid, Trail>,
    /// All categories by their full inheritance path name will store their indices in this map to be used by markers to find the cat index
    pub names_to_id_map: HashMap<String, usize>,
    /// This is what we will show to the user in terms of enabling/disabling categories and using this to adjust the currently drawn objects
    pub cat_selection_tree: Option<CatSelectionTree>,
}

impl MarkerPack {
    /// call this function to get a markerpack struct from a folder.
    pub fn new(folder_location: VfsPath, fm: &FileManager) -> Self {
        // our files in the markerpack directory
        let mut mfiles: Vec<MarkerFile> = Vec::new();
        let mut global_cats: Vec<IMCategory> = Vec::new();
        let mut global_pois: HashMap<Uuid, POI> = HashMap::new();
        let mut global_trails: HashMap<Uuid, Trail> = HashMap::new();
        let mut name_id_map: HashMap<String, usize> = HashMap::new();
        let mut cstree = vec![];

        let vid = fm.get_vid(&folder_location).unwrap();
        for f in folder_location
            .read_dir()
            .map_err(|e| {
                log::error!(
                    "couldn't open folder to read the entries. folder: {:?}, error: {:?}",
                    folder_location,
                    &e
                );
                e
            })
            .unwrap()
        {
            let entry = f;
            let ext = entry.extension();
            // for each xml file in this folder
            if ext == Some("xml".to_string()) {
                MarkerFile::parse_marker_file(
                    vid,
                    fm,
                    entry,
                    &mut global_cats,
                    &mut global_pois,
                    &mut global_trails,
                    &mut name_id_map,
                    &mut mfiles,
                    &mut cstree,
                );
            }
        }

        // insert uuids of markers and trails into global_cats so we can keep track of which markers to draw based on enabled categories.
        global_pois.values().for_each(|p| {
            p.register_category(&mut global_cats);
        });
        global_trails.values().for_each(|t| {
            t.register_category(&mut global_cats);
        });

        let cat_selection_tree = if cstree.is_empty() {
            None
        } else {
            Some(cstree.remove(0))
        };

        MarkerPack {
            path: vid,
            mfiles,
            global_cats,
            global_pois,
            global_trails,
            names_to_id_map: name_id_map,
            cat_selection_tree,
        }
    }

    pub fn update_active_markers(
        &self,
        mapid: u32,
        pack_index: usize,
        active_markers: &mut HashSet<(usize, usize, Uuid)>,
    ) {
        let mut active_cats = HashSet::new();
        if let Some(ref cstree) = self.cat_selection_tree {
            cstree.get_active_cats_indices(&mut active_cats);
        }
        for c in active_cats {
            for m in &self.global_cats[c.0].poi_registry {
                if let Some(p) = self.global_pois.get(&m) {
                    if p.map_id == mapid {
                        active_markers.insert((pack_index, c.0, *m));
                    }
                }
            }
        }
    }
    pub fn update_active_trails(
        &self,
        mapid: u32,
        pack_index: usize,
        active_trails: &mut HashSet<(usize, usize, Uuid)>,
    ) {
        let mut active_cats = HashSet::new();
        if let Some(ref cstree) = self.cat_selection_tree {
            cstree.get_active_cats_indices(&mut active_cats);
        }
        for c in active_cats {
            for id in &self.global_cats[c.0].trail_registry {
                if let Some(t) = self.global_trails.get(&id) {
                    if t.tdata.map_id == mapid {
                        active_trails.insert((pack_index, c.0, *id));
                    }
                }
            }
        }
    }
}
// /// represents a location in the tree where children are put in a vector
// #[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
// pub enum CatVecTree {
//     /// the top lvl Root node when you just start making up a tree
//     Root,
//     /// A non-root node which starts at root and goes through the children by a series of indices using the vector
//     /// the last index is the insert position in the vector pushing the rest by a place of one
//     Node(Vec<usize>)
// }

pub fn icon_file_to_vid(icon_path: &str, pack_path: VID, fm: &FileManager) -> Option<VID> {
        let pack_path = fm.get_path(pack_path).unwrap();
        let ipath = pack_path.join(icon_path).unwrap();
        if let Some(v) = fm.get_vid(&ipath) {
            Some(v)
        } else {
            log::error!(
                "{:?}, {:?}",
                ipath,
                pack_path,
            );
            None
        }

}
