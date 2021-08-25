use serde::{Deserialize, Serialize};
use vfs::{PhysicalFS, VfsPath};

/// File Manger to keep all the file/directory paths stored in one global place.
#[derive(Debug, Clone)]
pub struct FileManager {
    pub root: VfsPath,
    pub assets: VfsPath,
    pub markers: VfsPath,
    pub egui: VfsPath,
    pub paths: Vec<VfsPath>,
}
/// use VID to refer to these paths globally into the paths field of File Manager
#[derive(
    Debug, Clone, Copy, Serialize, Default, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct VID(pub usize);

impl FileManager {
    pub fn new() -> Self {
        let current_fs = PhysicalFS::new(std::env::current_dir().unwrap());
        let current_path = VfsPath::new(current_fs);
        assert!(current_path.exists().unwrap());
        let assets_path = current_path.join(JOKO_ASSET_FOLDER).unwrap();
        assert!(assets_path.exists().unwrap());
        let markers_path = assets_path.join(MARKER_PACK_FOLDER).unwrap();
        assert!(markers_path.exists().unwrap());
        let egui_path = assets_path.join(EGUI_TEXTURE_PATH).unwrap();
        let trail_path = assets_path.join(TRAIL_TEXTURE_PATH).unwrap();
        // assert!(egui_path.exists().unwrap());
        let mut paths = vec![];
        paths.push(egui_path.clone());
        paths.push(trail_path);
        for f in assets_path.walk_dir().unwrap() {
            let f = f.unwrap();
            paths.push(f);
        }

        Self {
            root: current_path,
            assets: assets_path,
            markers: markers_path,
            egui: egui_path,
            paths,
        }
    }
    pub fn get_vid(&self, path: &VfsPath) -> Option<VID> {
        self.paths.iter().position(|p| *p == *path).map(|p| VID(p))
    }
    pub fn get_path(&self, path_id: VID) -> Option<VfsPath> {
        self.paths.get(path_id.0).cloned()
    }
}

const JOKO_ASSET_FOLDER: &str = "assets";
const MARKER_PACK_FOLDER: &str = "packs";
const EGUI_TEXTURE_PATH: &str = "egui";
const TRAIL_TEXTURE_PATH: &str = "trail";