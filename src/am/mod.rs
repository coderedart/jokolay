use color_eyre::eyre::WrapErr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::Path;
use std::{fs::File, path::PathBuf};
use url::Url;

/// File Manger to keep all the file/directory paths stored in one global place.
#[derive(Debug, Clone)]
pub struct AssetManager {
    pub all_paths: Vec<PathBuf>,
    pub web_img_cache_map: HashMap<Url, usize>,
}


impl AssetManager {
    pub fn new(assets: PathBuf) -> Self {
        let assets_path = assets;
        if !assets_path.exists() {
            tracing::warn!("assets path doesn't exist. trying to create it.");

            create_dir_all(&assets_path).unwrap_or_else(|_| {
                panic!(
                    "failed to create assets path: {:#?}",
                    assets_path.as_os_str()
                )
            });
        }
        let markers_path = assets_path.join(MARKER_PACK_FOLDER_NAME);
        if !markers_path.exists() {
            tracing::warn!("marker packs path doesn't exist. trying to create it.");
            create_dir_all(&markers_path).unwrap_or_else(|_| {
                panic!(
                    "failed to create markers path: {:?}",
                    markers_path.as_os_str()
                )
            });
        }
        let log_file_path = assets_path.join(LOG_FILE_NAME);
        let egui_cache_path = assets_path.join(EGUI_CACHE_NAME);
        let config_file_path = assets_path.join(CONFIG_FILE_NAME);

        let web_img_cache_folder = assets_path.join(WEB_IMAGE_CACHE_FOLDER_NAME);
        if !web_img_cache_folder.exists() {
            tracing::warn!("web image cache folder path doesn't exist. trying to create it.");
            create_dir_all(&web_img_cache_folder).unwrap_or_else(|_| {
                panic!(
                    "failed to create web image cache folder: {:?}",
                    web_img_cache_folder.as_os_str()
                )
            });
        }
        let web_img_cache_map_file = web_img_cache_folder.join(WEB_IMAGE_CACHE_MAP_FILE_NAME);
        let marker_png_path = assets_path.join(MARKER_IMG_NAME);
        if !marker_png_path.exists() {
            tracing::warn!("marker img doesn't exist. trying to create it with default texture.");
            let marker_img = image::load_from_memory(MARKER_TEXTURE)
                .expect("failed to create image from default MARKER_TEXTURE");
            marker_img
                .save_with_format(&marker_png_path, image::ImageFormat::Png)
                .expect("failed to create marker.png");
        }
        let trail_png_path = assets_path.join(TRAIL_IMG_NAME);
        if !trail_png_path.exists() {
            tracing::warn!("trail img doesn't exist. trying to create it with default texture.");
            let trail_img = image::load_from_memory(TRAIL_TEXTURE)
                .expect("failed to create image from default TRAIL_TEXTURE");
            trail_img
                .save_with_format(&trail_png_path, image::ImageFormat::Png)
                .expect("failed to create trail.png");
        }
        let unknown_png_path = assets_path.join(UNKNOWN_IMG_NAME);
        if !unknown_png_path.exists() {
            tracing::warn!(
                "unknown (question) img doesn't exist. trying to create it with default texture."
            );
            let unknown_img = image::load_from_memory(QUESTION_TEXTURE)
                .expect("failed to create image from default QUESTION_TEXTURE");
            unknown_img
                .save_with_format(&unknown_png_path, image::ImageFormat::Png)
                .expect("failed to create unknown.png");
        }
        // IMPORTANT: make sure this matches the order of enums from above
        let all_paths = vec![
            assets_path,
            markers_path,
            log_file_path,
            config_file_path,
            egui_cache_path,
            web_img_cache_folder,
            web_img_cache_map_file,
            marker_png_path,
            trail_png_path,
            unknown_png_path,
        ];
        // let web_img_cache_map = Self::fill_web_cache_imgs(&mut all_paths);
        Self {
            all_paths,
            web_img_cache_map: Default::default(),
        }
    }
    pub fn fill_web_cache_imgs(_all_paths: &mut Vec<PathBuf>) -> HashMap<Url, usize> {
        todo!()
    }
    pub fn get_id_from_file_path(&self, path: &Path) -> color_eyre::Result<usize> {
        self.all_paths
            .iter()
            .position(|p| *p == *path)
            .wrap_err(format!("could not find path: {:?}", path.as_os_str()))
    }
    pub fn get_file_path_from_id(&self, id: usize) -> Option<&PathBuf> {
        self.all_paths.get(id)
    }
    pub fn get_id_from_url(&self, u: &Url) -> Option<&usize> {
        self.web_img_cache_map.get(u)
    }
    pub fn register_path(&mut self, path: PathBuf) -> usize {
        match self.all_paths.iter().position(|p| p == &path) {
            Some(index) => index,
            None => {
                let index = self.all_paths.len();
                self.all_paths.push(path);
                index
            }
        }
    }
    pub fn open_file(&self, id: usize) -> color_eyre::Result<File> {
        let path = self
            .get_file_path_from_id(id)
            .expect("invalid id given to open_file in AssetManager");
        Ok(File::open(path)?)
    }
}

pub const MARKER_PACK_FOLDER_NAME: &str = "packs";
pub const LOG_FILE_NAME: &str = "jokoloy.log";
pub const EGUI_CACHE_NAME: &str = "egui_cache.json";
pub const CONFIG_FILE_NAME: &str = "joko_config.json";

pub const ASSETS_FOLDER_NAME: &str = "assets";
pub const MARKER_IMG_NAME: &str = "marker.png";
pub const TRAIL_IMG_NAME: &str = "trail.png";
pub const UNKNOWN_IMG_NAME: &str = "unknown.png";

pub const WEB_IMAGE_CACHE_FOLDER_NAME: &str = "webcache";
pub const WEB_IMAGE_CACHE_MAP_FILE_NAME: &str = "webimgcache.json";

/// The default trail texture
const TRAIL_TEXTURE: &[u8] = include_bytes!("./trail.png");
/// The default Marker Texture
const MARKER_TEXTURE: &[u8] = include_bytes!("./marker.png");
/// The Question mark texture for when we can't find a texture
const QUESTION_TEXTURE: &[u8] = include_bytes!("./question.png");
