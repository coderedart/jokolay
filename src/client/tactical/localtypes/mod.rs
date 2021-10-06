use std::path::PathBuf;

use anyhow::Context;

use crate::client::am::AssetManager;

pub mod category;
pub mod files;
pub mod manager;
pub mod marker;
pub mod trail;
pub mod pack;

impl AssetManager {
    pub fn pack_relative_to_absolute_path(&self, pack_index: usize, path: &str) -> anyhow::Result<PathBuf> {
        Ok(self.get_file_path_from_id(pack_index).map(|pack_path| pack_path.join(path)).context("could not find pack path from id")?)
    }

}