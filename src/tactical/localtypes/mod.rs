pub mod marker;

use std::{collections::{BTreeMap, HashMap}, io::BufReader, path::PathBuf};
use quick_xml::de::from_reader as xmlreader;
use crate::tactical::{localtypes::marker::MarkerTemplate, xmltypes::{xml_category::{MarkerCategory, OverlayData}, xml_marker::POI, xml_trail::Trail}};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct CatSelectionTree {
    pub enabled: bool,
    pub children: Vec<CatSelectionTree>,
    pub id: usize,
    pub category_index: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkerFile {
    pub path: PathBuf,
    pub mc_index_tree: MCIndexTree,
    pub poi_vec: Vec<Uuid>,
    pub trl_vec: Vec<Uuid>,
    pub changes: Option<Vec<MarkerEditAction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCIndexTree {
    index: usize,
    children: Vec<MCIndexTree>,
}

#[derive(Debug, Clone,  )]
pub struct IMCategory {
    full_name: String,
    cat: MarkerCategory,
    inherited_template: MarkerTemplate,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkerPack {
    pub path: PathBuf,
    pub files: Vec<MarkerFile>,
    pub cats: Vec<IMCategory>,
    pub pois: BTreeMap<Uuid, POI>,
    pub trails: BTreeMap<Uuid, Trail>,
    pub names_to_id_map: HashMap<String, usize>,
    pub cat_selection_tree: CatSelectionTree,
}

impl MarkerPack {
    /// call this function everytime you modify the MarkerPack to update the category list
    pub fn new(zipfile_location: PathBuf) -> Self {
        //open the zip file
        let zipfile = std::fs::File::open(&zipfile_location).map_err(|e| {
            log::error!("couldn't open folder to read the entries. folder: {:?}, error: {:?}", zipfile_location, &e);
            e
        }).unwrap();
        let reader = BufReader::new(zipfile);
        let archive = zip::ZipArchive::new(reader).map_err(|e| {
            log::error!("couldn't open folder to read the entries. folder: {:?}, error: {:?}", zipfile_location, &e);
            e
        }).unwrap();
        // our files in the markerpack directory
        let mut files: Vec<MarkerFile> = Vec::new();
        let mut cats: Vec<IMCategory> = Vec::new();
        let mut pois = BTreeMap::new();
        let mut trails = BTreeMap::new();
        for f in archive.file_names() {
            
            if f.ends_with("xml") {
                let xml_file = archive.by_name(f).map_err(|e| {
                    log::error!("couldn't open xml file: {:?} due to error: {:?}", f, &e);
                    e
                }).unwrap();
                let marker_file_reader = std::io::BufReader::new(xml_file);

                let od: OverlayData = match xmlreader(marker_file_reader) {
                    Ok(od) => {
                       od
                    }
                    Err(e) => {
                        log::error!(
                            "failed to deserialize file {} due to error {:?}\n",
                            f,
                            e
                        );
                        continue;
                    }
                };
                let mut mc_index_tree ;
                if let Some(mctree) = od.categories {
                    
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
                id
            },
        }
    }

    // pub fn get_present_map_markers_with_inherit(&self, mapid: u32) -> Vec<POI> {
    //     let mut active_markers = vec![];
    //     let mut enabled_cats = Vec::new();
    //     {
    //         let root_template = self.cat_display.template.clone();
    //         CatTree::get_enabled_categories_with_inheritance(&self.cat_display.children, &mut enabled_cats, &root_template);
    //     }
    //     for pois in self.files.iter().map(|f| &f.od.pois) {
    //         if let Some(ref pois) = pois {
    //             if let Some(ref markers) = pois.poi {
    //                 let cat_parse = Instant::now();
    //                 for marker in markers.iter()
    //                 .filter(|&m| m.map_id == mapid)
    //                 .filter_map(|m| {
    //                     if let Some(cat_template) = enabled_cats.iter().find(|&c| c.name == m.category) {
    //                         let mut active_marker = m.clone();
    //                         active_marker.inherit_if_none(cat_template);
    //                         Some(active_marker)
    //                     } else {
    //                         None
    //                     }
    //                 }) {
    //                     active_markers.push(marker);
    //                 }
    //                 dbg!(cat_parse.elapsed());

    //             }
    //         }
    //     }

    //     active_markers
    // }
}
impl MCIndexTree {
    pub fn index_tree_from_mc_tree(mctree: Vec<MarkerCategory>, index_tree: &mut Vec<MCIndexTree>, cats: &mut Vec<IMCategory>, parent_template: MarkerTemplate, prefix: &str, name_id_map: &mut BTreeMap<String, usize>){
        for mc in mctree {
            let name = prefix.to_string() + &mc.name;
            if !name_id_map.contains_key(&name) {
                    let inherited_template = MarkerTemplate::default();
                    inherited_template.inherit_from_marker_category(&mc);
                    inherited_template.inherit_from_template(&parent_template);
                    let index = cats.len();
                    name_id_map.insert(name.clone(), index);
                    cats.push(IMCategory {
                        full_name: name,
                        cat: mc,
                        inherited_template, 
                    });                
            }
            let id: usize = name_id_map[&name];
           let mut children = Vec::new();
           if let Some(mc_children) = mc.children {
               Self::index_tree_from_mc_tree(mc_children, &mut children, cats, cats[id].inherited_template.clone(), &cats[id].full_name, name_id_map);
           }
            index_tree.push(MCIndexTree {
                index: id,
                children,
            });
        }
    }
}

// impl CatTree {
//     pub fn get_enabled_categories_with_inheritance(cats: &Vec<CatTree>, enabled_cats: &mut Vec<MarkerCategory>, parent_template: &MarkerCategory)  {
//         for cat in cats {
//             if cat.enabled.load(std::sync::atomic::Ordering::Relaxed) {
//                 let mut c = cat.template.clone();
//                 c.inherit_if_none(parent_template);
//                 c.name = parent_template.name.clone() + "." + &c.name;
//                 enabled_cats.push(c.clone());
//                 Self::get_enabled_categories_with_inheritance(&cat.children, enabled_cats, &c);

//             }
//         }
//     }
// }
// impl MarkerCategory {
//     pub fn register_category(clients: &mut Vec<MarkerCategory>, register: &mut Vec<CatTree>, id: &mut u32) {
//         for c in clients {
//             if let Some(existing_registration) =
//                 register.iter_mut().find(|r| r.template.name == c.name)
//             {
//                 // category already exists. good. maybe check if existing_category is equal to our new category
//                 if let Some(ref mut cc) = c.children {
//                     Self::register_category(cc, &mut existing_registration.children, id);
//                 }
//             } else {
//                 let template = c.clone();
//                 let enabled = Arc::new(AtomicBool::new(true));
//                 let mut children = Vec::new();
//                 if let Some(ref mut cc) = c.children {
//                     Self::register_category(cc, &mut children, id);
//                 }
//                 register.push(CatTree {
//                     template,
//                     enabled,
//                     children,
//                     id: *id
//                 });
//                 *id += 1;
//             }
//         }
//     }
// }

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
