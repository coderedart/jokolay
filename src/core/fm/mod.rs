use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use vfs::{PhysicalFS, VfsPath};

/// File Manger to keep all the file/directory paths stored in one global place.
#[derive(Debug, Clone)]
pub struct FileManager {
    pub assets: VfsPath,
    pub markers: VfsPath,
    pub paths: Vec<VfsPath>,
}
/// use VID to refer to these paths globally into the paths Vector field of File Manager
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RID {
    EguiTexture,
    MarkerTexture,
    TrailTexture,
    VID(usize),
}

impl FileManager {
    pub fn new(assets: PathBuf) -> Self {
        let assets_fs = PhysicalFS::new(assets);
        let assets_path = VfsPath::new(assets_fs);
        if !assets_path
            .exists()
            .map_err(|e| {
                log::error!(
                    "couldn't verify assets path existing due ot vfs error: {:#?}",
                    &e
                );
                e
            })
            .unwrap()
        {
            log::warn!("assets path doesn't exist. trying to create it.");
            assets_path
                .create_dir()
                .map_err(|e| {
                    log::error!("couldn't create assets path due to vfserror: {:#?}", &e);
                    e
                })
                .unwrap()
        }
        let markers_path = assets_path.join(MARKER_PACK_FOLDER_NAME).unwrap();
        if !markers_path
            .exists()
            .map_err(|e| {
                log::error!(
                    "couldn't verify marker packs path existing due ot vfs error: {:#?}",
                    &e
                );
                e
            })
            .unwrap()
        {
            log::warn!("marker packs path doesn't exist. trying to create it.");
            assets_path
                .create_dir()
                .map_err(|e| {
                    log::error!(
                        "couldn't create marker packs path due to vfserror: {:#?}",
                        &e
                    );
                    e
                })
                .unwrap()
        }
        let paths = vec![assets_path.clone(), markers_path.clone()];
        
        Self {
            assets: assets_path,
            markers: markers_path,
            paths,
        }
    }
    pub fn get_vid(&self, path: &VfsPath) -> Option<RID> {
        self.paths
            .iter()
            .position(|p| *p == *path)
            .map(|p| RID::VID(p))
            .or_else(|| {
                log::error!("could not find path: {}", path.as_str());
                None
            })
    }
    pub fn get_path(&self, vid: RID) -> Option<&VfsPath> {
        match vid {
            RID::VID(id) => self.paths.get(id),
            _ => unimplemented!(),
        }
    }
}

const MARKER_PACK_FOLDER_NAME: &str = "packs";
