//! How should the pack be stored by jokolay?
//! 1. Inside a directory called packs, we will have a separate directory for each pack.
//! 2. the name of the directory will serve as an ID for each pack.
//! 3. Inside the directory, we will have
//!     1. categories.xml -> The xml file which contains the whole category tree
//!     2. $mapid.xml -> where the $mapid is the id (u16) of a map which contains markers/trails belonging to that particular map.
//!     3. **/{.png | .trl} -> Any number of png images or trl binaries, in any location within this pack directory.

mod fs;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::Read,
    sync::{Arc, Mutex},
};

use cap_std::fs_utf8::Dir;
use egui::{CollapsingHeader, ColorImage, TextureHandle, Window};
use image::EncodableLayout;
use indexmap::IndexMap;
use tracing::{debug, error, info, info_span, warn_span};

use crate::{
    io::{load_pack_core_from_dir, save_pack_core_to_dir},
    pack::{Category, CommonAttributes, RelativePath},
};
use jokolink::MumbleLink;
use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

use super::pack::PackCore;

pub const PACK_LIST_URL: &str = "https://packlist.jokolay.com/packlist.json";

pub const MARKER_MANAGER_DIRECTORY_NAME: &str = "marker_manager";
pub const MARKER_PACKS_DIRECTORY_NAME: &str = "packs";
pub const CATEGORY_DATA_DIRECTORY_NAME: &str = "category_data";
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
    /// This contains the category selections of a marker pack. We store category selections separately so that if a pack is removed and added again, the selections stay the same
    category_data_dir: Dir,
    /// These are the marker packs
    /// The key is the name of the pack
    /// The value is a loaded pack that contains additional data for live marker packs like what needs to be saved or category selections etc..
    packs: BTreeMap<String, LoadedPack>,
    missing_texture: Option<TextureHandle>,
    /// This is the interval in number of seconds when we check if any of the packs need to be saved due to changes.
    /// This allows us to avoid saving the pack too often.
    pub save_interval: f64,
}
struct LoadedPack {
    /// The actual xml pack.
    pub core: PackCore,
    /// The selection of categories which are "enabled" and markers belonging to these may be rendered
    pub cats_selection: HashMap<String, CategorySelection>,
    /// whether cats selection needs to be saved
    pub cats_selection_dirty: bool,
    /// whether categories need to be saved
    pub cats: bool,
    /// Whether any mapdata needs saving
    pub map_dirty: HashSet<u32>,
    /// whether any texture needs saving
    pub texture: HashSet<RelativePath>,
    /// whether any tbin needs saving
    pub tbin: HashSet<RelativePath>,
    pub current_map_data: CurrentMapData,
}

#[derive(Default)]
pub struct CurrentMapData {
    /// the map to which the current map data belongs to
    pub map_id: u32,
    /// The textures that are being used by the markers, so must be kept alive by this hashmap
    pub active_textures: HashMap<RelativePath, TextureHandle>,
    pub active_markers: Vec<ActiveMarker>,
}

pub struct ActiveMarker {
    pub pos: glam::Vec3,
    pub width: u16,
    pub height: u16,
    pub texture: u64,
}
impl LoadedPack {
    fn tick(
        &mut self,
        etx: &egui::Context,
        _timestamp: f64,
        joko_renderer: &mut joko_render::JokoRenderer,
        link: &Option<Arc<MumbleLink>>,
        default_tex_id: &TextureHandle,
    ) {
        let link = match link {
            Some(link) => link,
            None => return,
        };
        for marker in self.current_map_data.active_markers.iter() {
            joko_renderer.add_billboard(marker.pos, marker.width, marker.height, marker.texture);
        }

        if self.current_map_data.map_id != link.map_id {
            info!(
                self.current_map_data.map_id,
                link.map_id, "current map data is updated."
            );
            if link.map_id == 0 {
                self.current_map_data = Default::default();
                return;
            }
            self.current_map_data.map_id = link.map_id;
            let mut enabled_cats_list = Default::default();
            CategorySelection::recursive_get_full_names(
                &self.cats_selection,
                &self.core.categories,
                &mut enabled_cats_list,
                "",
                &Default::default(),
            );
            for marker in self
                .core
                .maps
                .get(&link.map_id)
                .unwrap_or(&Default::default())
                .markers
                .iter()
            {
                if let Some(category_attributes) = enabled_cats_list.get(&marker.category) {
                    let mut common_attributes = marker.props.clone();
                    common_attributes.inherit_if_prop_none(category_attributes);
                    if let Some(tex_path) = &common_attributes.icon_file {
                        if !self.current_map_data.active_textures.contains_key(tex_path) {
                            if let Some(tex) = self.core.textures.get(tex_path) {
                                let img = image::load_from_memory(tex).unwrap();
                                self.current_map_data.active_textures.insert(
                                    tex_path.clone(),
                                    etx.load_texture(
                                        tex_path.as_str(),
                                        ColorImage::from_rgba_unmultiplied(
                                            [img.width() as _, img.height() as _],
                                            img.into_rgba8().as_bytes(),
                                        ),
                                        Default::default(),
                                    ),
                                );
                            } else {
                                info!(%tex_path, ?self.core.textures, "failed to find this texture");
                            }
                        }
                    } else {
                        info!(?marker.props.icon_file, "no texture attribute on this marker");
                    }
                    let th = common_attributes
                        .icon_file
                        .as_ref()
                        .map(|path| self.current_map_data.active_textures.get(path))
                        .flatten()
                        .unwrap_or(default_tex_id);
                    let (tex_id, width, height) = match th.id() {
                        egui::TextureId::Managed(tid) => {
                            (tid, th.size()[0] as u16, th.size()[1] as u16)
                        }
                        egui::TextureId::User(_) => unimplemented!(),
                    };
                    self.current_map_data.active_markers.push(ActiveMarker {
                        pos: marker.position,
                        width,
                        height,
                        texture: tex_id,
                    });
                } else {
                    info!(marker.category, ?marker.guid, "category is disabled. skipping marker");
                }
            }
        }
    }
}
#[derive(Debug, Default, Serialize, Deserialize)]
struct CategorySelection {
    pub selected: bool,
    pub display_name: String,
    pub children: HashMap<String, CategorySelection>,
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
        marker_manager_dir
            .create_dir_all(CATEGORY_DATA_DIRECTORY_NAME)
            .into_diagnostic()
            .wrap_err("failed to create category data dir")?;
        let category_data_dir = marker_manager_dir
            .open_dir(CATEGORY_DATA_DIRECTORY_NAME)
            .into_diagnostic()
            .wrap_err("failed to open category data dir")?;
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
                    match load_pack_core_from_dir(&pack_dir) {
                        Ok(pack_core) => {
                            let span_guard = info_span!("load category data", name).entered();
                            let category_data = category_data_dir.exists(&name).then(||  {
                                match category_data_dir.read_to_string(format!("{name}.json")) {
                                    Ok(cd_json) => {
                                        match serde_json::from_str(&cd_json) {
                                            Ok(cd) => Some(cd),
                                            Err(e) => {
                                                error!(?e, "failed to deserialize category data");
                                                None
                                            },
                                        }
                                    },
                                    Err(e) => {
                                        error!(?e, "failed to read string of category data");
                                        None
                                    },
                                }
                            }).flatten().unwrap_or_else(|| {
                                let cs = CategorySelection::default_from_pack_core(&pack_core);
                                match serde_json::to_string_pretty(&cs) {
                                    Ok(cs_json) => {
                                        match category_data_dir.write(format!("{name}.json"), &cs_json) {
                                            Ok(_) => {
                                                debug!("wrote category data {name}.json to disk after creating a default from pack");
                                            },
                                            Err(e) => {
                                                debug!(?e, "failed to write category data to disk");
                                            },
                                        }
                                    },
                                    Err(e) => {
                                        error!(?e, "failed to serialize cat selection");
                                    },
                                }
                                cs
                            });
                            std::mem::drop(span_guard);
                            packs.insert(
                                name.to_string(),
                                LoadedPack {
                                    core: pack_core,
                                    cats_selection: category_data,
                                    cats_selection_dirty: Default::default(),
                                    cats: Default::default(),
                                    map_dirty: Default::default(),
                                    texture: Default::default(),
                                    tbin: Default::default(),
                                    current_map_data: Default::default(),
                                },
                            );
                        }
                        Err(e) => {
                            error!(?e, "error while loading pack");
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
            category_data_dir,
            save_interval: 0.0,
            missing_texture: None,
        })
    }

    pub fn load() {}
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
        if timestamp - self.save_interval > 10.0 {
            self.save_interval = timestamp;
            for (pack_name, pack) in self.packs.iter_mut() {
                if pack.is_dirty() {
                    pack.save(
                        &pack_name,
                        &self.marker_packs_dir,
                        &self.category_data_dir,
                        false,
                    );
                }
            }
        }
        Window::new("Marker Manager").show(etx, |ui| -> Result<()> {
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

            CollapsingHeader::new("category selection").show(ui, |ui| {
                for (pack_name, pack) in self.packs.iter_mut() {
                    ui.menu_button(pack_name, |ui| {
                        CategorySelection::recursive_selection_ui( &mut pack.cats_selection, ui, &mut pack.cats_selection_dirty);
                    });
                }
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
                                ui.label("The marker pack is valid. please save it to complete the import process");
                                ui.horizontal(|ui| {
                                    ui.label("choose a pack name: ");    
                                    ui.text_edit_singleline(name);
                                });
                                let name = name.as_str();
                                ui.label("click \"save\" to save this pack to jokolay data directory.");
                                ui.colored_label(egui::Color32::YELLOW, "warning: If you don't click save, the import won't be complete");
                                if ui.button("save").clicked() {

                                    if self.marker_packs_dir.exists(name) {
                                        self.marker_packs_dir
                                            .remove_dir_all(name)
                                            .into_diagnostic()?;
                                    }
                                    let cats_selection = CategorySelection::default_from_pack_core(pack);
                                    let mut loaded_pack = LoadedPack {
                                        core: std::mem::take(pack),
                                        cats_selection ,
                                        cats_selection_dirty: true,
                                        cats: true,
                                        map_dirty: Default::default(),
                                        texture: Default::default(),
                                        tbin: Default::default(),
                                        current_map_data: Default::default()
                                    };
                                    loaded_pack.save(name, &self.marker_packs_dir, &self.category_data_dir, true);
                                    self.packs.insert(name.to_string(), loaded_pack);
                                    *saved = true;
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
impl CategorySelection {
    fn default_from_pack_core(pack: &PackCore) -> HashMap<String, CategorySelection> {
        let mut selection = HashMap::new();
        Self::recursive_create_category_selection(&mut selection, &pack.categories);
        selection
    }
    fn recursive_get_full_names(
        selection: &HashMap<String, CategorySelection>,
        cats: &IndexMap<String, Category>,
        list: &mut HashMap<String, CommonAttributes>,
        parent_name: &str,
        parent_common_attributes: &CommonAttributes,
    ) {
        for (name, cat) in cats {
            if let Some(selected_cat) = selection.get(name) {
                if !selected_cat.selected {
                    continue;
                }
                let full_name = if parent_name.is_empty() {
                    name.clone()
                } else {
                    format!("{parent_name}.{name}")
                };
                let mut common_attributes = cat.props.clone();
                common_attributes.inherit_if_prop_none(&parent_common_attributes);
                Self::recursive_get_full_names(
                    &selected_cat.children,
                    &cat.children,
                    list,
                    &full_name,
                    &common_attributes,
                );
                list.insert(full_name, common_attributes);
            }
        }
    }
    fn recursive_create_category_selection(
        selection: &mut HashMap<String, CategorySelection>,
        cats: &IndexMap<String, Category>,
    ) {
        for (cat_name, cat) in cats.iter() {
            let s = selection.entry(cat_name.clone()).or_default();
            s.selected = cat.default_enabled;
            s.display_name = cat.display_name.clone();
            Self::recursive_create_category_selection(&mut s.children, &cat.children);
        }
    }
    fn recursive_selection_ui(
        selection: &mut HashMap<String, CategorySelection>,
        ui: &mut egui::Ui,
        changed: &mut bool,
    ) {
        for (_cat_name, cat) in selection {
            ui.horizontal(|ui| {
                if ui.checkbox(&mut cat.selected, "").changed() {
                    *changed = true;
                }
                if !cat.children.is_empty() {
                    ui.menu_button(&cat.display_name, |ui: &mut egui::Ui| {
                        Self::recursive_selection_ui(&mut cat.children, ui, changed);
                    });
                } else {
                    ui.label(&cat.display_name);
                }
            });
        }
    }
}

impl LoadedPack {
    pub fn is_dirty(&self) -> bool {
        self.cats
            || self.cats_selection_dirty
            || !self.map_dirty.is_empty()
            || !self.texture.is_empty()
            || !self.tbin.is_empty()
    }
    #[tracing::instrument(skip(self, marker_packs_dir, category_data_dir))]
    pub fn save(&mut self, name: &str, marker_packs_dir: &Dir, category_data_dir: &Dir, all: bool) {
        if let Err(e) = marker_packs_dir.create_dir_all(name) {
            error!(?e, "failed to create directory for pack");
            return;
        }
        let pack_dir = match marker_packs_dir.open_dir(name) {
            Ok(d) => d,
            Err(e) => {
                error!(?e, "failed to open marker pack directory to save pack");
                return;
            }
        };
        if self.cats_selection_dirty || all {
            match serde_json::to_string_pretty(&self.cats_selection) {
                Ok(cs_json) => match category_data_dir.write(format!("{name}.json"), &cs_json) {
                    Ok(_) => {
                        debug!(
                            name,
                            "wrote category data to disk after creating a default from pack"
                        );
                    }
                    Err(e) => {
                        error!(?e, "failed to write category data to disk");
                    }
                },
                Err(e) => {
                    error!(?e, "failed to serialize cat selection");
                }
            }
        }
        match save_pack_core_to_dir(
            &self.core,
            &pack_dir,
            self.cats,
            std::mem::take(&mut self.map_dirty),
            std::mem::take(&mut self.texture),
            std::mem::take(&mut self.tbin),
            all,
        ) {
            Ok(_) => {
                debug!(name, "saved pack to directory");
            }
            Err(e) => {
                error!(?e, "failed to save pack to directory");
            }
        }
    }
}
