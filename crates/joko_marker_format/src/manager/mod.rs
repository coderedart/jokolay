use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
};

use joko_core::prelude::{egui::Window, *};

use crate::io::{load_pack_core_from_dir, save_pack_core_to_dir};

use super::pack::PackCore;

pub const PACK_LIST_URL: &str = "https://packlist.jokolay.com/packlist.json";

pub const MARKER_MANAGER_DIRECTORY_NAME: &str = "marker_manager";
pub const MARKER_PACKS_DIRECTORY_NAME: &str = "packs";

/// How should the pack be stored by jokolay?
/// 1. Inside a directory called packs, we will have a separate directory for each pack.
/// 2. the name of the directory will serve as an ID for each pack.
/// 3. Inside the directory, we will have
///     1. categories.xml -> The xml file which contains the whole category tree
///     2. $mapid.xml -> where the $mapid is the id (u16) of a map which contains markers/trails belonging to that particular map.
///     3. **/{.png | .trl} -> Any number of png images or trl binaries, in any location within this pack directory.
pub struct MarkerManager {
    pub ui_data: MarkerManagerUI,
    pub marker_manager_dir: Dir,
    pub marker_packs_dir: Dir,
    pub packs: BTreeMap<String, PackCore>,
}

#[derive(Debug, Default)]
pub struct MarkerManagerUI {
    pub import_pack_name: String,
    // tf is this type supposed to be? maybe we should have used a ECS for this reason.
    pub import_status: Option<Arc<Mutex<Option<Result<(String, PackCore)>>>>>,
}

#[derive(Debug, Default)]
pub struct PackList {
    pub packs: BTreeMap<String, PackEntry>,
}

#[derive(Debug)]
pub struct PackEntry {
    pub url: Url,
    pub description: String,
}

impl MarkerManager {
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
        let mut packs: BTreeMap<String, PackCore> = Default::default();

        for entry in marker_packs_dir
            .entries()
            .into_diagnostic()
            .wrap_err("failed to get entries of marker packs dir")?
        {
            let entry = entry.into_diagnostic()?;
            if entry.metadata().into_diagnostic()?.is_file() {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                let pack_dir = entry
                    .open_dir()
                    .into_diagnostic()
                    .wrap_err("failed to open pack entry as directory")?;
                {
                    let span_guard = warn_span!("loading pack from dir", name).entered();
                    match load_pack_core_from_dir(&pack_dir) {
                        Ok(pack_core) => {
                            packs.insert(name.to_string(), pack_core);
                        }
                        Err(e) => {
                            error!("error while loading pack: {e}");
                        }
                    }
                    drop(span_guard);
                }
            }
        }

        Ok(Self {
            packs,
            marker_packs_dir,
            marker_manager_dir,
            ui_data: Default::default(),
        })
    }

    pub fn load() {}

    pub fn tick(&mut self, etx: &egui::Context, _timestamp: f64) {
        Window::new("Marker Manager").show(etx, |ui| -> Result<()> {
            if ui.button("import pack").clicked() {
                let import_status = Arc::new(Mutex::default());
                self.ui_data.import_status = Some(import_status.clone());
                rayon::spawn(move || {
                    if let Some(file_path) = rfd::FileDialog::new()
                        .add_filter("taco", &["zip", "taco"])
                        .pick_file()
                    {
                        let result = import_pack_from_zip_file_path(file_path);
                        import_status
                            .lock()
                            .expect("failed to lock imported pack mutex")
                            .replace(result);
                    }
                });
            }
            if self.ui_data.import_status.is_some() {
                if ui.button("clear").clicked() {
                    self.ui_data.import_status = None;
                }
            }
            if let Some(import_status) = self.ui_data.import_status.as_ref() {
                if let Ok(status) = import_status.try_lock() {
                    if let Some(status) = status.as_ref() {
                        match status {
                            Ok((name, pack)) => {
                                ui.horizontal(|ui| {
                                    ui.label("name: ");
                                    ui.label(name);
                                });
                                if ui.button("save this pack").clicked() {
                                    if self.marker_packs_dir.exists(name) {
                                        self.marker_packs_dir
                                            .remove_dir_all(name)
                                            .into_diagnostic()?;
                                    }
                                    self.marker_packs_dir.create_dir(name).into_diagnostic()?;
                                    save_pack_core_to_dir(
                                        pack,
                                        &self.marker_packs_dir.open_dir(name).into_diagnostic()?,
                                    )?;
                                }
                            }
                            Err(e) => {
                                ui.colored_label(
                                    egui::Color32::RED,
                                    format!("failed to import pack due to error: {e}"),
                                );
                            }
                        }
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "pack is being imported");
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
