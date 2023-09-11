//! How should the pack be stored by jokolay?
//! 1. Inside a directory called packs, we will have a separate directory for each pack.
//! 2. the name of the directory will serve as an ID for each pack.
//! 3. Inside the directory, we will have
//!     1. categories.xml -> The xml file which contains the whole category tree
//!     2. $mapid.xml -> where the $mapid is the id (u16) of a map which contains markers/trails belonging to that particular map.
//!     3. **/{.png | .trl} -> Any number of png images or trl binaries, in any location within this pack directory.

mod live_pack;
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
};

use cap_std::fs_utf8::Dir;
use egui::{CollapsingHeader, ColorImage, TextureHandle, Window};
use image::EncodableLayout;

use tracing::{error, info, info_span};

use jokolink::MumbleLink;
use miette::{Context, IntoDiagnostic, Result};

use self::live_pack::LoadedPack;

use super::pack::PackCore;

pub const PACK_LIST_URL: &str = "https://packlist.jokolay.com/packlist.json";

pub const MARKER_MANAGER_DIRECTORY_NAME: &str = "marker_manager";
pub const MARKER_PACKS_DIRECTORY_NAME: &str = "packs";
pub const MARKER_MANAGER_CONFIG_NAME: &str = "marker_manager_config.json";
/// It manage everything that has to do with marker packs.
/// 1. imports, loads, saves and exports marker packs.
/// 2. maintains the categories selection data for every pack
/// 3. contains activation data globally and per character
/// 4. When we load into a map, it filters the markers and runs the logic every frame
///     1. If a marker needs to be activated (based on player position or whatever)
///     2. marker needs to be drawn
///     3. marker's texture is uploaded or being uploaded? if not ready, we will upload or use a temporary "loading" texture
///     4. render that marker use joko_render  
pub struct MarkerManager {
    /// holds data that is useful for the ui
    ui_data: MarkerManagerUI,
    /// marker manager directory. not useful yet, but in future we could be using this to store config files etc..
    _marker_manager_dir: Dir,
    /// packs directory which contains marker packs. each directory inside pack directory is an individual marker pack.
    /// The name of the child directory is the name of the pack
    marker_packs_dir: Dir,
    /// These are the marker packs
    /// The key is the name of the pack
    /// The value is a loaded pack that contains additional data for live marker packs like what needs to be saved or category selections etc..
    packs: BTreeMap<String, LoadedPack>,
    missing_texture: Option<TextureHandle>,
    /// This is the interval in number of seconds when we check if any of the packs need to be saved due to changes.
    /// This allows us to avoid saving the pack too often.
    pub save_interval: f64,
}

#[derive(Debug, Default)]
pub enum ImportStatus {
    #[default]
    UnInitialized,
    WaitingForFileChooser,
    LoadingPack(std::path::PathBuf),
    PackDone(String, PackCore, bool),
    PackError(miette::Report),
}
#[derive(Debug, Default)]
pub struct MarkerManagerUI {
    pub import_pack_name: String,
    // tf is this type supposed to be? maybe we should have used a ECS for this reason.
    pub import_status: Option<Arc<Mutex<ImportStatus>>>,
}

#[derive(Debug, Default)]
pub struct PackList {
    pub packs: BTreeMap<String, PackEntry>,
}

#[derive(Debug)]
pub struct PackEntry {
    pub url: url::Url,
    pub description: String,
}

impl MarkerManager {
    /// Creates a new instance of [MarkerManager].
    /// 1. It opens the marker manager directory
    /// 2. loads its configuration
    /// 3. opens the packs directory
    /// 4. loads all the packs
    /// 5. loads all the activation data
    /// 6. returns self
    pub fn new(jdir: &Dir) -> Result<Self> {
        jdir.create_dir_all(MARKER_MANAGER_DIRECTORY_NAME)
            .into_diagnostic()
            .wrap_err("failed to create marker manager directory")?;
        let marker_manager_dir = jdir
            .open_dir(MARKER_MANAGER_DIRECTORY_NAME)
            .into_diagnostic()
            .wrap_err("failed to open marker manager directory")?;
        marker_manager_dir
            .create_dir_all(MARKER_PACKS_DIRECTORY_NAME)
            .into_diagnostic()
            .wrap_err("failed to create marker packs directory")?;
        let marker_packs_dir = marker_manager_dir
            .open_dir(MARKER_PACKS_DIRECTORY_NAME)
            .into_diagnostic()
            .wrap_err("failed to open marker packs dir")?;
        let mut packs: BTreeMap<String, LoadedPack> = Default::default();

        for entry in marker_packs_dir
            .entries()
            .into_diagnostic()
            .wrap_err("failed to get entries of marker packs dir")?
        {
            let entry = entry.into_diagnostic()?;
            if entry.metadata().into_diagnostic()?.is_file() {
                continue;
            }
            if let Ok(name) = entry.file_name() {
                let pack_dir = entry
                    .open_dir()
                    .into_diagnostic()
                    .wrap_err("failed to open pack entry as directory")?;
                {
                    let span_guard = info_span!("loading pack from dir", name).entered();
                    match LoadedPack::load_from_dir(pack_dir) {
                        Ok(lp) => {
                            packs.insert(name, lp);
                        }
                        Err(e) => {
                            error!(?e, "failed to load pack from directory");
                        }
                    }
                    drop(span_guard);
                }
            }
        }

        Ok(Self {
            packs,
            marker_packs_dir,
            _marker_manager_dir: marker_manager_dir,
            ui_data: Default::default(),
            save_interval: 0.0,
            missing_texture: None,
        })
    }

    fn pack_importer(import_status: Arc<Mutex<ImportStatus>>) {
        rayon::spawn(move || {
            *import_status.lock().unwrap() = ImportStatus::WaitingForFileChooser;

            if let Some(file_path) = rfd::FileDialog::new()
                .add_filter("taco", &["zip", "taco"])
                .pick_file()
            {
                *import_status.lock().unwrap() = ImportStatus::LoadingPack(file_path.clone());

                let result = import_pack_from_zip_file_path(file_path);
                match result {
                    Ok((name, pack)) => {
                        *import_status.lock().unwrap() = ImportStatus::PackDone(name, pack, false);
                    }
                    Err(e) => {
                        *import_status.lock().unwrap() = ImportStatus::PackError(e);
                    }
                }
            } else {
                *import_status.lock().unwrap() =
                    ImportStatus::PackError(miette::miette!("file chooser was cancelled"));
            }
        });
    }
    pub fn tick(
        &mut self,
        etx: &egui::Context,
        timestamp: f64,
        joko_renderer: &mut joko_render::JokoRenderer,
        link: &Option<Arc<MumbleLink>>,
    ) {
        if self.missing_texture.is_none() {
            let img = image::load_from_memory(include_bytes!("../pack/marker.png")).unwrap();
            let size = [img.width() as _, img.height() as _];
            self.missing_texture = Some(etx.load_texture(
                "default marker",
                ColorImage::from_rgba_unmultiplied(size, img.into_rgba8().as_bytes()),
                egui::TextureOptions {
                    magnification: egui::TextureFilter::Linear,
                    minification: egui::TextureFilter::Linear,
                },
            ));
        }

        for pack in self.packs.values_mut() {
            pack.tick(
                etx,
                timestamp,
                joko_renderer,
                link,
                self.missing_texture.as_ref().unwrap(),
            );
        }
    }
    pub fn menu_ui(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Markers", |ui| {
            for pack in self.packs.values_mut() {
                pack.category_sub_menu(ui);
            }
        });
    }
    pub fn gui(&mut self, etx: &egui::Context, open: &mut bool) {
        Window::new("Marker Manager").open(open).show(etx, |ui| -> Result<()> {
            CollapsingHeader::new("Loaded Packs").show(ui, |ui| {
                egui::Grid::new("packs").striped(true).show(ui, |ui| {
                    let mut delete = vec![];
                for pack in self.packs.keys() {
                    ui.label(pack);
                    if ui.button("delete").clicked() {
                        delete.push(pack.clone());
                    }
                }
                for pack_name in delete {
                    self.packs.remove(&pack_name);
                    if let Err(e) = self.marker_packs_dir.remove_dir_all(&pack_name) {
                        error!(?e, pack_name,"failed to remove pack");
                    } else {
                        info!("deleted marker pack: {pack_name}");
                    }
                }
            });
            });

            if self.ui_data.import_status.is_some() {
                if ui.button("clear").on_hover_text(
                    "This will cancel any pack import in progress. If import is already finished, then it wil simply clear the import status").clicked() {
                    self.ui_data.import_status = None;
                }
            } else {
                if ui.button("import pack").on_hover_text("select a taco/zip file to import the marker pack from").clicked() {
                    let import_status = Arc::new(Mutex::default());
                    self.ui_data.import_status = Some(import_status.clone());
                    Self::pack_importer(import_status);
                }
            }
            if let Some(import_status) = self.ui_data.import_status.as_ref() {
                if let Ok(mut status) = import_status.lock() {
                    match &mut *status {
                        ImportStatus::UnInitialized => {
                            ui.label("import not started yet");
                        }
                        ImportStatus::WaitingForFileChooser => {
                            ui.label(
                                "wailting for the file dialog. choose a taco/zip file to import",
                            );
                        }
                        ImportStatus::LoadingPack(p) => {
                            ui.label(format!("pack is being imported from {p:?}"));
                        }
                        ImportStatus::PackDone(name, pack, saved) => {

                            if !*saved {
                                ui.horizontal(|ui| {
                                    ui.label("choose a pack name: ");    
                                    ui.text_edit_singleline(name);
                                });
                                let name = name.as_str();
                                if ui.button("save").clicked() {

                                    if self.marker_packs_dir.exists(name) {
                                        self.marker_packs_dir
                                            .remove_dir_all(name)
                                            .into_diagnostic()?;
                                    }
                                    if let Err(e) = self.marker_packs_dir.create_dir_all(name) {
                                        error!(?e, "failed to create directory for pack");

                                    }
                                    match self.marker_packs_dir.open_dir(name) {
                                        Ok(dir) => {
                                            let core = std::mem::take(pack);
                                            let mut loaded_pack = LoadedPack::new(core, dir);
                                            match loaded_pack.save_all() {
                                                Ok(_) => {
                                                    self.packs.insert(name.to_string(), loaded_pack);
                                                    *saved = true;
                                                },
                                                Err(e) => {
                                                    error!(?e, "failed to save marker pack");
                                                },
                                            }
                                        },
                                        Err(e) => {
                                            error!(?e, "failed to open marker pack directory to save pack");
                                        }
                                    };
                                }
                            } else {
                                ui.colored_label(egui::Color32::GREEN, "pack is saved. press click `clear` button to remove this message");
                            }
                        }
                        ImportStatus::PackError(e) => {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("failed to import pack due to error: {e:#?}"),
                            );
                        }
                    }
                }
            }

            Ok(())
        });
    }
}

fn import_pack_from_zip_file_path(file_path: std::path::PathBuf) -> Result<(String, PackCore)> {
    let mut taco_zip = vec![];
    std::fs::File::open(&file_path)
        .into_diagnostic()?
        .read_to_end(&mut taco_zip)
        .into_diagnostic()?;

    info!("starting to get pack from taco");
    crate::io::get_pack_from_taco_zip(&taco_zip).map(|pack| {
        (
            file_path
                .file_name()
                .map(|ostr| ostr.to_string_lossy().to_string())
                .unwrap_or_default(),
            pack,
        )
    })
}
