use std::{collections::HashSet, fs::read_dir, path::{Path, PathBuf}};

use uuid::Uuid;

use crate::tactical::localtypes::MarkerPack;

/// Manages all the marker packs including loading and storing them. 
pub struct MarkerManager {
    /// folder which contains all the marker packs as direct subdirectories.
    pub location: PathBuf,
    /// the MarkerPacks which were created from the subfolders of the location
    pub packs: Vec<MarkerPack>,
    /// whether we should draw the markers. useful to control the rendering of the markers
    pub draw_markers: bool,
    /// the active_markers set with a tuple of (markerpack index, ImCat index inside the marker pack, Uuid of the marker/trail). passing marker manager
    /// to renderer, it can use this and the packs field to construct the marker nodes to draw. 
    pub active_markers: HashSet<(usize, usize, Uuid)>,
    /// the state for egui but not directly useful for the markermanager 
    pub state: EState,
}
/// state required for egui, but not necessarily useful for the core struct itself. 
#[derive(Debug, Clone, Default)]
pub struct EState {
    pub load_folder_path: String,
    pub show_cat_selection_window: bool,

}
impl MarkerManager {
    /// tries to create MarkerPacks from each directory in the specified location and return a new MarkerManger .
    pub fn new(location: &Path) -> Self {
        let mut packs = vec![];
        let entries = read_dir(&location)
            .map_err(|e| {
                log::error!(
                    "couldn't open folder to read the entries. folder: {:?}, error: {:?}",
                    location,
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

            if entry.file_type().unwrap().is_dir() {
                packs.push(MarkerPack::new(entry.path()));
            }
        }
        Self {
            location: location.to_path_buf(),
            packs,
            draw_markers: false,
            active_markers: HashSet::new(),
            state: EState {
                load_folder_path: location.to_str().unwrap_or_default().to_string(),
                show_cat_selection_window: false,

            }
        }
    }
}