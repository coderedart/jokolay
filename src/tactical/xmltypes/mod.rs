use std::{
    ffi::OsStr,
    fs::read_dir,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

use crate::tactical::xmltypes::{
    xml_category::{MarkerCategory, OverlayData},
    xml_marker::POI,
};

use serde::{Deserialize, Serialize};
//use serde_xml_rs::from_reader as xmlreader;
use quick_xml::de::from_reader as xmlreader;
use {
    xml_category::MarkerCategory as XMLCategory,
};

pub mod xml_category;
pub mod xml_marker;
pub mod xml_trail;

#[derive(Debug, Default)]
pub struct CatTree {
    pub template: XMLCategory,
    pub enabled: Arc<AtomicBool>,
    pub children: Vec<CatTree>,
    pub id: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkerFile {
    pub path: PathBuf,
    pub od: OverlayData,
    pub changes: Option<Vec<MarkerEditAction>>,
}

#[derive(Debug)]
pub struct MarkerPack {
    pub files: Vec<MarkerFile>,
    pub cat_display: CatTree,
}

impl MarkerPack {
    /// call this function everytime you modify the MarkerPack to update the category list
    pub fn new(folder_location: &PathBuf) -> Self {
        let mut id = 0u32;
        // our files in the markerpack directory
        let mut files: Vec<MarkerFile> = Vec::new();
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
                let marker_file_reader = std::io::BufReader::new(xml_file);

                match xmlreader(marker_file_reader) {
                    Ok(od) => {
                        files.push(MarkerFile {
                            path: entry.path(),
                            od,
                            changes: None,
                        });
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
        // the root category template
        let mut template: Option<MarkerCategory> = None;
        let enabled = Arc::new(AtomicBool::new(true));
        // the children vector to fill the sub categories into
        let mut children = Vec::new();
        // for each file
        for f in files.iter_mut() {
            // if file has MarkerCategories
            if let Some(ref mut mc) = f.od.categories {
                // if root category already init, check that the present file has the same root category
                if let Some(ref mut t) = template {
                    assert_eq!(t.name, mc.name);
                } else {
                    // if this is the first time we got a category file, set the root category
                    template = Some(mc.clone());
                }
                // build/merge/add the subcategories to the catdisplay tree IF it has children

                if let Some(ref mut cc) = mc.children {
                    MarkerCategory::register_category(cc, &mut children, &mut id);
                }
            }
        }
        MarkerPack {
            files,
            cat_display: CatTree {
                template: template.unwrap_or(MarkerCategory::default()),
                enabled,
                children,
                id,
            },
        }
    }

    pub fn get_present_map_markers_with_inherit(&self, mapid: u32) -> Vec<POI> {
        let mut active_markers = vec![];
        let mut enabled_cats = Vec::new();
        {
            let root_template = self.cat_display.template.clone();
            CatTree::get_enabled_categories_with_inheritance(
                &self.cat_display.children,
                &mut enabled_cats,
                &root_template,
            );
        }
        for pois in self.files.iter().map(|f| &f.od.pois) {
            if let Some(ref pois) = pois {
                if let Some(ref markers) = pois.poi {
                    let cat_parse = Instant::now();
                    for marker in markers
                        .iter()
                        .filter(|&m| m.map_id == mapid)
                        .filter_map(|m| {
                            if let Some(cat_template) =
                                enabled_cats.iter().find(|&c| c.name == m.category)
                            {
                                let mut active_marker = m.clone();
                                active_marker.inherit_if_none(cat_template);
                                Some(active_marker)
                            } else {
                                None
                            }
                        })
                    {
                        active_markers.push(marker);
                    }
                    dbg!(cat_parse.elapsed());
                }
            }
        }

        active_markers
    }
}

impl CatTree {
    pub fn get_enabled_categories_with_inheritance(
        cats: &Vec<CatTree>,
        enabled_cats: &mut Vec<MarkerCategory>,
        parent_template: &MarkerCategory,
    ) {
        for cat in cats {
            if cat.enabled.load(std::sync::atomic::Ordering::Relaxed) {
                let mut c = cat.template.clone();
                c.inherit_if_none(parent_template);
                c.name = parent_template.name.clone() + "." + &c.name;
                enabled_cats.push(c.clone());
                Self::get_enabled_categories_with_inheritance(&cat.children, enabled_cats, &c);
            }
        }
    }
}
impl MarkerCategory {
    pub fn register_category(
        clients: &mut Vec<MarkerCategory>,
        register: &mut Vec<CatTree>,
        id: &mut u32,
    ) {
        for c in clients {
            if let Some(existing_registration) =
                register.iter_mut().find(|r| r.template.name == c.name)
            {
                // category already exists. good. maybe check if existing_category is equal to our new category
                if let Some(ref mut cc) = c.children {
                    Self::register_category(cc, &mut existing_registration.children, id);
                }
            } else {
                let template = c.clone();
                let enabled = Arc::new(AtomicBool::new(true));
                let mut children = Vec::new();
                if let Some(ref mut cc) = c.children {
                    Self::register_category(cc, &mut children, id);
                }
                register.push(CatTree {
                    template,
                    enabled,
                    children,
                    id: *id,
                });
                *id += 1;
            }
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
