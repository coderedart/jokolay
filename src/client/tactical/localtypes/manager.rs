

use anyhow::Context;
use tokio::fs::read_dir;

use uuid::Uuid;

use crate::client::{am::{AssetManager, AssetPaths}, tactical::localtypes::{category::{CategoryIndex, IMCategory}, files::MarkerFile, marker::POI, pack::MarkerPack, trail::Trail}};


/// Manages all the marker packs including loading and storing them.
pub struct MarkerManager {
    pub categories: Vec<IMCategory>,
    pub markers: Vec<POI>,
    pub trails: Vec<Trail>,
    pub marker_files: Vec<MarkerFile>,
    /// the MarkerPacks which were created from the subfolders of the location
    pub packs: Vec<MarkerPack>,
    /// whether we should draw the markers. useful to control the rendering of the markers
    pub draw_markers: bool,
}
/// state required for egui, but not necessarily useful for the core struct itself.
#[derive(Debug)]
pub struct EState {
    pub active_cats_changed: bool,
    pub current_map: u32,
    pub load_folder_path: String,
    pub show_cat_selection_window: bool,
    pub info_window: bool,
    pub show_editor_window: bool,
    pub editor: Editor,
}

#[derive(Debug)]
pub struct Editor {
    pub selected_pack: usize,
    pub selected_category: CategoryIndex,
    pub selected_marker: Uuid,
}

#[derive(Debug)]
pub struct CategoryEditor {
    pub active_pack: usize,
    pub active_category: CategoryIndex,
}

#[derive(Debug)]
pub struct MarkerEditor {
    pub active_pack: usize,
    pub active_marker: Uuid,
}
#[derive(Debug)]
pub struct MarkerFileEditor {
    pub active_pack: usize,
    pub active_file: usize,
}
impl MarkerManager {
    /// tries to create MarkerPacks from each directory in the specified location and return a new MarkerManger .
    pub async fn new(am: &mut AssetManager) -> anyhow::Result<Self> {
        use num_traits::ToPrimitive;

        let mut categories: Vec<IMCategory> = vec![];
        let mut markers: Vec<POI> = vec![];
        let mut trails: Vec<Trail> = vec![];
        let mut marker_files: Vec<MarkerFile> = vec![];
        let mut packs = vec![];
        let location = am.get_file_path_from_id(AssetPaths::MarkerPacks.to_usize().expect("failed to convert MarkerPacks enum to usize")).cloned().expect("failed to get marker packs location");
        let mut entries = read_dir(&location).await?;
        while let Some(dir) = entries.next_entry().await?
        {
            let dir_type = dir.metadata().await.context(format!("failed to get metadata about folder: {:?}", &dir))?;
            if dir_type.is_dir() {
                let pack_path_id = am.register_path(dir.path());
                packs.push(MarkerPack::new(pack_path_id, am, &mut categories, &mut markers, &mut trails, &mut marker_files).await?);
            }
        }
        Ok(Self {
            categories,
            markers,
            trails,
            marker_files,
            packs,
            draw_markers: false,
        })
    }
}
