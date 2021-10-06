use crate::client::{
    am::AssetManager,
    tactical::{localtypes::marker::MarkerTemplate, xmltypes::xml_category::XMLMarkerCategory},
};

// use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// The primary abstraction for marker category. Inherited Marker Category.
#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct IMCategory {
    /// The full inherited name to match against the `type` field of a POI
    pub full_name: String,
    /// The original Category to save back to a Marker File
    pub cat: XMLMarkerCategory,
    /// The template to inherit from for "effectively" inherity from all the parent categrories and this category.
    /// using this we avoid writing the inherited fields to `cat` field itself and keep it clean to be written back to overlaydata.
    pub inherited_template: MarkerTemplate,
    /// this field contains a list of all the POI Uuids which matched against this category's full_name. easier to keep track of markers belonging to a particular category.
    pub poi_registry: Vec<Uuid>,
    /// list of Trail tag guids
    pub trail_registry: Vec<Uuid>,
}

/// The struct represents a category selection of a particular marker pack and the category index/id are only valid for that marker pack.
/// This is used to primarily remember which categories are enabled and also show as a category selection widget in egui for users to enable categories
/// by using category_index, we keep the struct small and also allows for categories to be referenced globally.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct CatSelectionTree {
    pub enabled: bool,
    pub children: Vec<CatSelectionTree>,
    pub state: Estate,
    pub category_index: CategoryIndex,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Estate {
    pub id: usize,
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
                    state: Estate {
                        id: fastrand::usize(..usize::MAX),
                    },
                    category_index: mci.index,
                })
            }
        }
    }
    /// gets the indexes of active IMCats and then we can query the active markers from those cats to build up a list of markers to draw.
    pub fn get_active_cats_indices(&self, active_cats: &mut HashSet<CategoryIndex>) {
        if self.enabled {
            active_cats.insert(self.category_index);
            for cs in &self.children {
                cs.get_active_cats_indices(active_cats);
            }
        }
    }
}

/// A MarkerCategory Tree representation using only the indexes of the categories stored in the global cats of the pack.
/// useful to derive cat selection tree and also to write back to a markerfile/overlaydata exactly as it was before.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCIndexTree {
    pub index: CategoryIndex,
    pub children: Vec<MCIndexTree>,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, PartialOrd, Eq, Ord, Hash,
)]
pub struct CategoryIndex(pub usize);

impl MCIndexTree {
    /// This function takes a raw xml MarkerCategory tree and recursively puts them in the global cats, stores their indices in index_tree, and finally puts their full names in the name_id_map
    pub fn index_tree_from_mc_tree(
        pack_path: usize,
        am: &mut AssetManager,
        raw_xmlmc_tree: Vec<XMLMarkerCategory>,
        index_tree: &mut Vec<MCIndexTree>,
        cats: &mut Vec<IMCategory>,
        parent_template: MarkerTemplate,
        prefix: &str,
        name_id_map: &mut HashMap<String, usize>,
    ) -> anyhow::Result<()> {
        for mut mc in raw_xmlmc_tree {
            // this checks if this is the root/top of the category tree.
            let name = if !prefix.is_empty() {
                // if this is a child, take the parent's full name and attach its own name after the pullstop to make its fullname
                prefix.to_string() + "." + &mc.name
            } else {
                // if this is the root, its own name is the fullname
                mc.name.clone()
            };
            // take the children of this MC node to parse them
            let mc_children = mc.children.take();
            // if this cat name has not been seen in the global name_id_map, then we add it
            if !name_id_map.contains_key(&name) {
                let mut inherited_template = MarkerTemplate::default();
                let mut icon_path_id = None;
                // if icon file has path
                if let Some(ref icon_path) = mc.icon_file {
                    // get absolute path
                    let ipath = am.pack_relative_to_absolute_path(pack_path, &icon_path)?;
                        // get the id of the absolute path
                    icon_path_id = Some(am.get_id_from_file_path(&ipath)?);
                    
                }

                inherited_template.inherit_from_marker_category(&mc, icon_path_id);
                inherited_template.inherit_from_template(&parent_template);
                let index = cats.len();
                cats.push(IMCategory {
                    full_name: name.clone(),
                    cat: mc,
                    inherited_template,
                    poi_registry: vec![],
                    trail_registry: vec![],
                });
                name_id_map.insert(name.clone(), index);
            }
            let id: usize = name_id_map[&name];
            let mut children = Vec::new();
            if let Some(mc_children) = mc_children {
                Self::index_tree_from_mc_tree(
                    pack_path,
                    am,
                    mc_children,
                    &mut children,
                    cats,
                    cats[id].inherited_template.clone(),
                    &name,
                    name_id_map,
                )?;
            }
            index_tree.push(MCIndexTree {
                index: CategoryIndex(id),
                children,
            });
        }
        Ok(())
    }
}
