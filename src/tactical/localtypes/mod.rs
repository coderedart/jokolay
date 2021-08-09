pub mod marker;
pub mod manager;
use crate::tactical::{
    localtypes::marker::MarkerTemplate,
    xmltypes::{
        xml_category::{MarkerCategory, OverlayData},
        xml_marker::POI,
        xml_trail::Trail,
    },
};
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

/// The struct represents a category selection of a particular marker pack and the category index/id are only valid for that marker pack.
/// This is used to primarily remember which categories are enabled and also show as a category selection widget in egui for users to enable categories
/// by using category_index, we keep the struct small and also allows for categories to be referenced globally. 
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct CatSelectionTree {
    pub enabled: bool,
    pub children: Vec<CatSelectionTree>,
    pub id: usize,
    pub category_index: usize,
}

impl CatSelectionTree {
    /// builds a category selection tree recursively. the category index is the index of a category in the global ImCategory vector.
    /// MCIndexTree is primarily just to organize the categories into a tree struct so that we can build cstree from it
    pub fn build_cat_selection_tree(
        mc_index_tree: &Vec<MCIndexTree>,
        cstree: &mut Vec<CatSelectionTree>,
    ) {
        for mci in mc_index_tree {
            if let Some(existing_cat) = cstree.iter_mut().find(|c| c.category_index == mci.index) {
                Self::build_cat_selection_tree(&mci.children, &mut existing_cat.children);
            } else {
                let mut children = vec![];
                Self::build_cat_selection_tree(&mci.children, &mut children);
                cstree.push(CatSelectionTree {
                    enabled: true,
                    children,
                    id: fastrand::usize(..usize::MAX),
                    category_index: mci.index,
                })
            }
        }
    }
    /// gets the indexes of active IMCats and then we can query the active markers from those cats to build up a list of markers to draw.
    pub fn get_active_cats_indices(&self, active_cats: &mut HashSet<usize>) {
        if self.enabled {
            active_cats.insert(self.category_index);
            for cs in &self.children {
                cs.get_active_cats_indices(active_cats);
            }
        }
    }
}
/// Marker File is the primary abstraction to use for editing markerpacks. they can only have one Active Marker File to edit.
/// it is a abstract representation of the `OverlayData` struct which we deserialize from xml files. this helps us keep all the markers/cats in one place
/// while also have this struct represent the OverlayData struct with just their Uuids/indexes. when we want to create/edit an existing marker file, this is 
/// the primary struct to use as it will remember the path of the file and overwrite it with the changes when necessary. 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkerFile {
    pub path: PathBuf,
    pub mc_index_tree: Option<MCIndexTree>,
    pub poi_vec: Vec<Uuid>,
    pub trl_vec: Vec<Uuid>,
    pub changes: Option<Vec<MarkerEditAction>>,
}

/// A MarkerCategory Tree representation using only the indexes of the categories stored in the global cats of the pack. 
/// useful to derive cat selection tree and also to write back to a markerfile/overlaydata exactly as it was before.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCIndexTree {
    pub index: usize,
    pub children: Vec<MCIndexTree>,
}

/// The primary abstraction for marker category. 
#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct IMCategory {
    /// The full inherited name to match against the `type` field of a POI 
    pub full_name: String,
    /// The original Category to save back to a Marker File
    pub cat: MarkerCategory,
    /// The template to inherit from for "effectively" inherity from all the parent categrories and this category.
    /// using this we avoid writing the inherited fields to `cat` field itself and keep it clean to be written back to overlaydata.
    pub inherited_template: MarkerTemplate,
    /// this field contains a list of all the POI Uuids which matched against this category's full_name. easier to keep track of markers belonging to a particular category.
    pub poi_registry: Vec<Uuid>,
}
/// Zip Crate is getting a new API overhaul soon. so, until then just use normal forlders. The pack itself should be self-contained including the images/other file references relative to this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkerPack {
    /// The path to the folder where the marker xml files and other data live.
    pub path: PathBuf,
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
    pub fn new(folder_location: PathBuf) -> Self {
        // our files in the markerpack directory
        let mut files: Vec<MarkerFile> = Vec::new();
        let mut global_cats: Vec<IMCategory> = Vec::new();
        let mut global_pois: HashMap<Uuid, POI> = HashMap::new();
        let mut global_trails: HashMap<Uuid, Trail> = HashMap::new();
        let mut name_id_map: HashMap<String, usize> = HashMap::new();
        let mut cstree = vec![];
        let entries = read_dir(&folder_location)
            .map_err(|e| {
                log::error!(
                    "couldn't open folder to read the entries. folder: {:?}, error: {:?}",
                    folder_location,
                    &e
                );
                e
            })
            .unwrap();
        for f in entries {
            let entry = f
                .map_err(|e| {
                    log::error!("couldn't read entry. error: {:?}", &e);
                    e
                })
                .unwrap();

            if entry.path().extension() == Some(OsStr::new("xml")) {
                let xml_file = std::fs::File::open(&entry.path())
                    .map_err(|e| {
                        log::error!(
                            "couldn't open xml file: {:?} due to error: {:?}",
                            &entry.path(),
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
                            &entry,
                            e
                        );
                        continue;
                    }
                };
                let mut mc_index_tree = Vec::new();
                if let Some(mctree) = od.categories {
                    MCIndexTree::index_tree_from_mc_tree(
                        vec![mctree],
                        &mut mc_index_tree,
                        &mut global_cats,
                        MarkerTemplate::default(),
                        "",
                        &mut name_id_map,
                    )
                }
                let mut uuid_poi_vec = Vec::new();
                let mut uuid_trail_vec = Vec::new();
                if let Some(p) = od.pois {
                    if let Some(vp) = p.poi {
                        uuid_poi_vec = POI::get_vec_uuid_pois(vp, &mut global_pois);
                    }
                    if let Some(vt) = p.trail {
                        uuid_trail_vec = Trail::get_vec_uuid_trail(vt, &mut global_trails);
                    }
                }
                CatSelectionTree::build_cat_selection_tree(&mc_index_tree, &mut cstree);
                let mc_index_tree = if !mc_index_tree.is_empty() {
                    Some(mc_index_tree.remove(0))
                } else {
                    None
                };
                files.push(MarkerFile {
                    path: entry.path(),
                    mc_index_tree,
                    poi_vec: uuid_poi_vec,
                    trl_vec: uuid_trail_vec,
                    changes: None,
                });
            }
        }
        global_pois.values().for_each(|p| {
            p.register_category(&mut global_cats);
        });
        let cat_selection_tree = if cstree.is_empty() {
            None
        } else {
            Some(cstree.remove(0))
        };
        MarkerPack {
            path: folder_location,
            mfiles: files,
            global_cats,
            global_pois,
            global_trails,
            names_to_id_map: name_id_map,
            cat_selection_tree,
        }
    }

    pub fn fill_muuid_cindex_map(&self, mapid: u32, pack_index: usize, active_markers: &mut HashSet<(usize, usize, Uuid)>) {
        let mut active_cats = HashSet::new();
        if let Some(ref cstree) = self.cat_selection_tree {
            cstree.get_active_cats_indices(&mut active_cats);
        }
        for c in active_cats {
            for m in &self.global_cats[c].poi_registry {
                if let Some(p) = self.global_pois.get(&m) {
                    if p.map_id == mapid {
                        active_markers.insert((pack_index, c, *m));
                    }
                }
            }
        }
    }
}
impl MCIndexTree {
    pub fn index_tree_from_mc_tree(
        mctree: Vec<MarkerCategory>,
        index_tree: &mut Vec<MCIndexTree>,
        cats: &mut Vec<IMCategory>,
        parent_template: MarkerTemplate,
        prefix: &str,
        name_id_map: &mut HashMap<String, usize>,
    ) {
        for mut mc in mctree {
            let name = if !prefix.is_empty() {
                prefix.to_string() + "." + &mc.name
            } else {
                mc.name.clone()
            };
            let mc_children = mc.children.take();
            if !name_id_map.contains_key(&name) {
                let mut inherited_template = MarkerTemplate::default();
                inherited_template.inherit_from_marker_category(&mc);
                inherited_template.inherit_from_template(&parent_template);
                let index = cats.len();
                name_id_map.insert(name.clone(), index);
                cats.push(IMCategory {
                    full_name: name.clone(),
                    cat: mc,
                    inherited_template,
                    poi_registry: vec![],
                });
            }
            let id: usize = name_id_map[&name];
            let mut children = Vec::new();
            if let Some(mc_children) = mc_children {
                Self::index_tree_from_mc_tree(
                    mc_children,
                    &mut children,
                    cats,
                    cats[id].inherited_template.clone(),
                    &name,
                    name_id_map,
                );
            }
            index_tree.push(MCIndexTree {
                index: id,
                children,
            });
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum MarkerEditAction {
    CreateCategory {
        location: Vec<usize>,
        new_category: MarkerCategory,
    },
    UpdateCategory {
        location: Vec<usize>,
        previous: MarkerCategory,
        updated: MarkerCategory,
    },
    DeleteCategory {
        location: Vec<usize>,
        deleted_item: MarkerCategory,
    },
    CreateMarker {
        location: usize,
        new_marker: POI,
    },
    UpdateMarker {
        location: usize,
        previous: POI,
        updated: POI,
    },
    DeleteMarker {
        location: usize,
        deleted_item: POI,
    },
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
