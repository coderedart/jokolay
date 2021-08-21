use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    fm::{FileManager, VID},
    tactical::localtypes::MarkerPack,
};

/// Manages all the marker packs including loading and storing them.
pub struct MarkerManager {
    /// folder which contains all the marker packs as direct subdirectories.
    pub path: VID,
    /// the MarkerPacks which were created from the subfolders of the location
    pub packs: Vec<MarkerPack>,
    /// whether we should draw the markers. useful to control the rendering of the markers
    pub draw_markers: bool,
    /// the active_markers set with a tuple of (markerpack index, ImCat index inside the marker pack, Uuid of the marker/trail). passing marker manager
    /// to renderer, it can use this and the packs field to construct the marker nodes to draw.
    pub active_markers: HashSet<(usize, usize, Uuid)>,
    /// active trails from enabled categories similar to active_markers.
    pub active_trails: HashSet<(usize, usize, Uuid)>,
    /// the state for egui but not directly useful for the markermanager
    pub state: EState,
}
/// state required for egui, but not necessarily useful for the core struct itself.
#[derive(Debug, Clone, Default)]
pub struct EState {
    pub active_cats_changed: bool,
    pub load_folder_path: String,
    pub show_cat_selection_window: bool,
}
impl MarkerManager {
    /// tries to create MarkerPacks from each directory in the specified location and return a new MarkerManger .
    pub fn new(fm: &FileManager) -> Self {
        let mut packs = vec![];
        let location = fm.markers.clone();
        for dir in location
            .read_dir()
            .map_err(|e| {
                log::error!(
                    "couldn't open folder to read the entries. folder: {:?}, error: {:?}",
                    location,
                    &e
                );
                e
            })
            .unwrap()
        {
            let dir_type = dir.metadata().unwrap();
            if dir_type.file_type == vfs::path::VfsFileType::Directory {
                packs.push(MarkerPack::new(dir, fm));
            }
        }
        let vid = fm.get_vid(&location).unwrap();
        Self {
            path: vid,
            packs,
            draw_markers: false,
            active_markers: HashSet::new(),
            active_trails: HashSet::new(),
            state: EState {
                active_cats_changed: true,
                load_folder_path: location.as_str().to_string(),
                show_cat_selection_window: false,
            },
        }
    }
}
