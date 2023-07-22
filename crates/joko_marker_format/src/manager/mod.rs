use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use cap_std::fs::Dir;
use egui::{
    epaint::ahash::{HashMap, HashSet},
    DragValue, Window,
};
use indexmap::IndexMap;
use miette::{Context, Diagnostic, IntoDiagnostic, Result};

use serde::{Deserialize, Serialize};

use time::OffsetDateTime;

use tracing::warn;
use uuid::Uuid;

use crate::pack::PackInfo;

use super::pack::Pack;

pub const PACK_LIST_URL: &str = "https://packlist.jokolay.com/packlist.json";
/// How should the pack be stored by jokolay?
/// 1. Inside a directory called packs, we will have a separate directory for each pack.
/// 2. the name of the directory will be a random UUID, which will serve as an ID for each pack locally.
/// 3. Inside the directory, we will have
///     1. categories.xml -> The xml file which contains the whole category tree
///     2. $mapid.xml -> where the $mapid is the id (u16) of a map which contains markers/trails belonging to that particular map.
///     3. **/{.png | .trl} -> Any number of png images or trl binaries, in any random directories within this pack directory.
///     4. info.json -> pack info like name, version, the url from which it was downloaded by jokolay or if it was manually imported from zip file locally etc..
///     5. activation.json -> list of categories which are enabled, list of markers which had their behavior triggered.
/// 4. This will allow many packs with same name /version. This will allow people to duplicate a pack and edit it, while preserving the source/original pack as is.
pub struct MarkerManager {
    pub marker_manager_dir: Dir,
    pub marker_packs_dir: Dir,
    pub last_update_attempt: f64,
    pub pack_list: Arc<Mutex<PackList>>,
    pub packs: BTreeMap<Uuid, LivePack>,
    pub packs_being_downloaded: BTreeMap<String, Arc<Mutex<PackDownloadStatus>>>,
    // pub ui_data: MarkerManagerUIData,
    pub number_of_markers_to_draw: usize,
}

pub struct MarkerManagerConfig {}

pub enum PackDownloadStatus {
    Downloading,
    Converting,
    Saving,
    Done,
    Failed(String),
}

// #[derive(Debug, Default)]
// pub struct MarkerManagerUIData {
//     selected_pack: String,
//     selected_map: u16,
//     selected_marker: usize,
//     _selected_trail: usize,
// }

pub struct LivePack {
    /// This is the directory of this pack. Any texture or whatever will have to be relative to this path.
    dir: Dir,
    /// This is the marker pack data. we reuse the struct as xml, but one crucial difference is that, this is usually only loaded partially.
    /// As images only need to be loaded as needed.
    pack: Pack,
    /// Activation data stored inside the pack dir as well.
    activation_data: ActivationData,
    /// Info about the pack
    info: PackInfo,
}

impl LivePack {
    pub fn new(dir: Dir) -> Result<Self> {
        let pack = Pack::from_dir(&dir).wrap_err("failed to load marker pack from directory")?;

        let activation_data = dir.read_to_string("activation.json").unwrap_or_default();
        let activation_data = serde_json::from_str(&activation_data).unwrap_or_default();
        let info = dir.read_to_string("info.json").unwrap_or_default();
        let info = serde_json::from_str(&info).unwrap_or_default();
        Ok(Self {
            pack,
            activation_data,
            dir,
            info,
        })
    }
    pub fn get_pack(&self) -> &Pack {
        &self.pack
    }
    pub fn save_everything(&self) -> miette::Result<()> {
        Ok(())
    }
    pub fn save_trigger_data(&self) {}
}
#[derive(Debug, Default)]
pub struct LiveMarkers {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActivationData {
    /// If the category full name is in this set, then it is enabled.
    pub enabled_cats: HashSet<String>,
    /// u16 key is the map id.
    /// The value is a map of all the marker ids which are "triggered" and will wake up at the timestamp value
    pub sleeping_markers: HashMap<u16, IndexMap<Uuid, OffsetDateTime>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PackList {
    pub packs: BTreeMap<String, PackEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackEntry {
    pub version: semver::Version,
    pub url: url::Url,
    pub description: String,
}

impl MarkerManager {
    const MARKER_MANAGER_PATH: &str = "marker_manager";
    const MARKER_PACKS_PATH: &str = "packs";
    /// MarkerManager needs the zip files to load as markers and data stored like activation data or enabled categories.
    /// 1.
    pub fn new(jdir: &Dir) -> Result<Self> {
        jdir.create_dir_all(Self::MARKER_MANAGER_PATH)
            .into_diagnostic()
            .wrap_err("failed to create marker manager directory")?;
        let marker_manager_dir = jdir
            .open_dir(Self::MARKER_MANAGER_PATH)
            .into_diagnostic()
            .wrap_err("failed to open marker manager directory")?;
        marker_manager_dir
            .create_dir_all(Self::MARKER_PACKS_PATH)
            .into_diagnostic()
            .wrap_err("failed to create marker packs directory")?;
        let marker_packs_dir = marker_manager_dir
            .open_dir(Self::MARKER_PACKS_PATH)
            .into_diagnostic()
            .wrap_err("failed ot open marker packs dir")?;
        let packs: BTreeMap<Uuid, LivePack> = Default::default();

        for entry in marker_packs_dir
            .entries()
            .into_diagnostic()
            .wrap_err("failed to get entries of marker packs dir")?
        {
            let entry = entry.into_diagnostic()?;
            if entry.metadata().into_diagnostic()?.is_file() {
                continue;
            }
            if let Some(_name) = entry.file_name().to_str() {
                // let name: Uuid = name
                //     .parse()
                //     .into_diagnostic()
                //     .wrap_err("pack name is not valid utf-8")?;

                // // entry.open_dir().into_diagnostic()?.open(path)
                // activation_data_path.set_file_name("name.adata");
                // let live_pack = LivePack::new(pack.path(), activation_data_path)?;
                // packs.insert(name, live_pack);
            }
        }
        let pack_list = Arc::new(Mutex::default());

        Ok(Self {
            pack_list,
            packs,
            last_update_attempt: 0.0,
            packs_being_downloaded: BTreeMap::new(),
            marker_packs_dir,
            marker_manager_dir,
            number_of_markers_to_draw: 100,
        })
    }

    pub fn load() {}
    // pub fn render(&self, map_id: u16, renderer: &mut JokoRenderer) {
    //     renderer.markers.clear();
    //     let camera_position = renderer.camera_position;
    //     for pack in self.packs.values() {
    //         for marker in pack
    //             .pack
    //             .markers
    //             .values()
    //             .filter(|m| {
    //                 m.map_id == map_id
    //                     && m.position.distance(camera_position) < MARKER_MAX_VISIBILITY_DISTANCE
    //             })
    //             .take(self.number_of_markers_to_draw)
    //         {
    //             //     if let Some(texture) = marker.props.texture {
    //             //     if renderer.textures.contains_key(texture) {
    //             //         renderer.draw_marker(joko_render::MarkerQuad {
    //             //             position: marker.position,
    //             //             texture: marker.texture.into(),
    //             //             width: pack.pack.textures[marker.texture as usize].width,
    //             //             height: pack.pack.textures[marker.texture as usize].height,
    //             //         });
    //             //     } else {
    //             //         let png = pack.pack.textures.get(marker.texture as usize).unwrap();
    //             //         let img = image::load_from_memory(&png.bytes).unwrap();
    //             //         let pixels = img.into_rgba8().into_vec();
    //             //         renderer.upload_texture(
    //             //             marker.texture as u32,
    //             //             png.width as u32,
    //             //             png.height as u32,
    //             //             pixels,
    //             //         )
    //             //     }
    //             // }
    //         }
    //     }
    // }
    pub fn tick(&mut self, etx: &egui::Context, timestamp: f64) {
        Window::new("Marker Manager").show(etx, |ui| -> miette::Result<()> {
            ui.add(DragValue::new(&mut self.number_of_markers_to_draw));
            if ui.button("import pack").clicked() {
                let name = Uuid::new_v4();
                self.marker_packs_dir
                    .create_dir(format!("{name}"))
                    .into_diagnostic()?;

                let dir = self
                    .marker_packs_dir
                    .open_dir(format!("{name}"))
                    .into_diagnostic()?;
                tokio::spawn(async move {
                    if let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("taco", &["zip", "taco"])
                        .pick_file()
                        .await
                    {
                        let taco_zip = file.read().await;
                        warn!("starting to get pack from taco");
                        match Pack::get_pack_from_taco_zip(&taco_zip) {
                            Ok(pack) => {
                                pack.save_to_dir(&dir).unwrap();
                                warn!("saved pack");
                            }
                            Err(_e) => {}
                        }
                    }
                });
            }
            egui::CollapsingHeader::new("Pack List ")
                .default_open(false)
                .show(ui, |ui| {
                    ui.label(format!(
                        "last packlist update attempt: {} seconds ago",
                        (timestamp - self.last_update_attempt) as u64
                    ));
                    if ui.button("update list").clicked() {
                        let pack_list = self.pack_list.clone();
                        tokio::task::spawn(async move {
                            let newlist: PackList = reqwest::get(PACK_LIST_URL)
                                .await
                                .into_diagnostic()
                                .unwrap()
                                .json()
                                .await
                                .into_diagnostic()
                                .unwrap();
                            *pack_list.lock().unwrap() = newlist;
                        });
                        self.last_update_attempt = timestamp;
                    }

                    if let Some(plist) = self.pack_list.lock().ok() {
                        for (pack_name, pack_entry) in plist.packs.iter() {
                            ui.group(|ui| -> miette::Result<()> {
                                if ui.button("Install").clicked() {
                                    let status =
                                        Arc::new(Mutex::new(PackDownloadStatus::Downloading));
                                    let url = pack_entry.url.clone();
                                    let _version = pack_entry.version.clone();
                                    let name = Uuid::new_v4();
                                    self.packs_being_downloaded
                                        .insert(pack_name.clone(), status.clone());
                                    self.marker_packs_dir
                                        .create_dir(format!("{name}"))
                                        .into_diagnostic()?;
                                    let dir = self
                                        .marker_packs_dir
                                        .open_dir(format!("{name}"))
                                        .into_diagnostic()?;
                                    tokio::task::spawn(async move {
                                        let xmlpack = match reqwest::get(url.as_str()).await {
                                            Ok(response) => match response.bytes().await {
                                                Ok(bytes) => bytes,
                                                Err(e) => {
                                                    *status.lock().unwrap() =
                                                        PackDownloadStatus::Failed(e.to_string());
                                                    return;
                                                }
                                            },
                                            Err(e) => {
                                                *status.lock().unwrap() =
                                                    PackDownloadStatus::Failed(e.to_string());
                                                return;
                                            }
                                        };
                                        warn!("starting to get pack from taco");
                                        match Pack::get_pack_from_taco_zip(&xmlpack) {
                                            Ok(pack) => {
                                                // warn!("failures when converting pack ({name}) to json:\n{failures:#?}");
                                                pack.save_to_dir(&dir).unwrap();
                                                *status.lock().unwrap() = PackDownloadStatus::Done;
                                                warn!("saved pack");
                                            }
                                            Err(e) => {
                                                *status.lock().unwrap() =
                                                    PackDownloadStatus::Failed(e.to_string());
                                            }
                                        }
                                    });
                                }
                                ui.horizontal(|ui| {
                                    ui.label("id: ");
                                    ui.label(pack_name.as_str());
                                });
                                ui.horizontal(|ui| {
                                    ui.label("description: ");
                                    ui.label(pack_entry.description.as_str());
                                });

                                ui.label(format!("version: {}", pack_entry.version));

                                ui.horizontal(|ui| {
                                    ui.label("url: ");
                                    ui.label(pack_entry.url.as_str());
                                });
                                Ok(())
                            });
                        }
                    }
                });
            Ok(())
        });
    }
}
