use anyhow::Context;
use std::{
    collections::{HashMap},
};
use tokio::fs::read_dir;


use crate::client::{
    am::AssetManager,
    tactical::{
        localtypes::{
            category::{CatSelectionTree, IMCategory},
            files::MarkerFile,
            marker::POI,
            trail::Trail,
        },
    },
};

///  The pack itself should be self-contained including the images/other file references relative to this. Zip Crate is getting a new API overhaul soon. so, until then just use normal forlders.
#[derive(Debug, Clone)]
pub struct MarkerPack {
    /// The path to the folder where the marker xml files and other data live.
    pub path: usize,
    /// the marker files collected so that we can later just turn them back into overlaydata if we have changes.
    pub mfiles: Vec<usize>,
    /// list of all the images that are in this folder, this will help with selecting the texture for a marker/trail
    pub images: Vec<usize>,
    /// list of all the trail files that are in this pack, will help to edit these seperately from markers
    pub trail_files: Vec<usize>,
    /// This is what we will show to the user in terms of enabling/disabling categories and using this to adjust the currently drawn objects
    pub cat_selection_tree: Option<CatSelectionTree>,
}

impl MarkerPack {
    /// call this function to get a markerpack struct from a folder.
    pub async fn new(
        pack_path_id: usize,
        am: &mut AssetManager,
        global_cats: &mut Vec<IMCategory>,
        global_pois: &mut Vec<POI>,
        global_trails: &mut Vec<Trail>,
        global_marker_files: &mut Vec<MarkerFile>,
    ) -> anyhow::Result<MarkerPack> {
        let mut name_id_map: HashMap<String, usize> = HashMap::new();
        let mut cstree = vec![];
        let pack_path = am
            .get_file_path_from_id(pack_path_id)
            .context("pack folder not present in AssetManager")?
            .clone();

        let mut xml_files = vec![];
        let mut images = vec![];
        let mut trail_files = vec![];
        let mut entries = read_dir(&pack_path).await?;
        while let Some(entry) = entries.next_entry().await.context(format!(
            "failed to read dir entries of pack: {:?}",
            &pack_path
        ))? {
            let entry_type = entry
                .file_type()
                .await
                .context(format!("failed to get filetype of {:?}", &entry))?;
            if entry_type.is_file() {
                let file_name = entry
                    .file_name()
                    .to_str()
                    .context(format!("failed to convert filename to str: {:?}", &entry))?
                    .to_string();
                let ext = entry
                    .path()
                    .extension()
                    .context(format!("failed to get extension of file: {:?}", &file_name))?
                    .to_str()
                    .context(format!(
                        "failed to convert extension to unicode: {:?}",
                        &entry
                    ))?
                    .to_string();
                if file_name.starts_with(".") {
                    continue;
                }
                match ext.as_str() {
                            "xml" => {
                                let marker_file_id = am.register_path(entry.path());
                                xml_files.push(marker_file_id);
                            },
                            "png" => {
                                let image_id = am.register_path(entry.path());
                                images.push(image_id);
                            },
                            "trl" => {
                                let trail_data_id = am.register_path(entry.path());
                                trail_files.push(trail_data_id);
                            },
                            _ => log::warn!("file with extension that is not png, trl or xml found in pack folder {} with filename: {:?}", entry.path().display(), pack_path)
                        }
            }
        }
        let mut mfiles = vec![];
        for xml_file_index in xml_files {
            let _xml_file = am
                .get_file_path_from_id(xml_file_index)
                .context(format!("failed to get xml file from id"))?;
            let mfile = MarkerFile::new(
                pack_path_id,
                xml_file_index,
                am,
                global_cats,
                global_pois,
                global_trails,
                &mut name_id_map,
                &mut cstree,
            )
            .await?;
            let index = global_marker_files.len();
            global_marker_files.push(mfile);
            mfiles.push(index);
        }

        global_pois.iter_mut().for_each(|p| {
            p.register_category(global_cats);
        });
        global_trails.iter_mut().for_each(|t| {
            t.register_category(global_cats);
        });

        let cat_selection_tree = if cstree.is_empty() {
            None
        } else {
            Some(cstree.remove(0))
        };

        Ok(MarkerPack {
            path: pack_path_id,
            mfiles,
            cat_selection_tree,
            images,
            trail_files,
        })
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
