use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use cap_std::fs_utf8::Dir;
use egui::{ColorImage, TextureHandle};
use glam::{vec2, Vec2, Vec3};
use image::EncodableLayout;
use indexmap::IndexMap;
use joko_render::billboard::{MarkerObject, MarkerVertex, TrailObject};
use tracing::{debug, error, info};

use crate::{
    io::{load_pack_core_from_dir, save_pack_core_to_dir},
    pack::{Category, CommonAttributes, PackCore, RelativePath},
    INCHES_PER_METER,
};
use jokolink::MumbleLink;
use miette::{bail, Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

pub struct LoadedPack {
    /// The directory inside which the pack data is stored
    /// There should be a subdirectory called `core` which stores the pack core
    /// Files related to Jokolay thought will have to be stored directly inside this directory, to keep the xml subdirectory clean.
    /// eg: Active categories, activation data etc..
    pub dir: Dir,
    /// The actual xml pack.
    pub core: PackCore,
    /// The selection of categories which are "enabled" and markers belonging to these may be rendered
    cats_selection: HashMap<String, CategorySelection>,
    dirty: Dirty,
    current_map_data: CurrentMapData,
}

#[derive(Debug, Default, Clone)]
struct Dirty {
    all: bool,
    /// whether categories need to be saved
    cats: bool,
    /// whether cats selection needs to be saved
    cats_selection: bool,
    /// Whether any mapdata needs saving
    map_dirty: HashSet<u32>,
    /// whether any texture needs saving
    texture: HashSet<RelativePath>,
    /// whether any tbin needs saving
    tbin: HashSet<RelativePath>,
}

impl Dirty {
    fn is_dirty(&self) -> bool {
        self.cats
            || self.cats_selection
            || !self.map_dirty.is_empty()
            || !self.texture.is_empty()
            || !self.tbin.is_empty()
    }
}

impl LoadedPack {
    const CORE_PACK_DIR_NAME: &str = "core";
    const CATEGORY_SELECTION_FILE_NAME: &str = "cats.json";
    pub fn new(core: PackCore, dir: Dir) -> Self {
        let cats_selection = CategorySelection::default_from_pack_core(&core);
        LoadedPack {
            core,
            cats_selection,
            dirty: Dirty {
                all: true,
                ..Default::default()
            },
            current_map_data: Default::default(),
            dir,
        }
    }
    pub fn category_sub_menu(&mut self, ui: &mut egui::Ui) {
        CategorySelection::recursive_selection_ui(
            &mut self.cats_selection,
            ui,
            &mut self.dirty.cats_selection,
        );
    }
    pub fn load_from_dir(dir: Dir) -> Result<Self> {
        if !dir
            .try_exists(Self::CORE_PACK_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to check if pack core exists")?
        {
            bail!("pack core doesn't exist in this pack");
        }
        let core_dir = dir
            .open_dir(Self::CORE_PACK_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to open core pack directory")?;
        let core = load_pack_core_from_dir(&core_dir).wrap_err("failed to load pack from dir")?;

        let cats_selection = (match dir.read_to_string(Self::CATEGORY_SELECTION_FILE_NAME) {
            Ok(cd_json) => match serde_json::from_str(&cd_json) {
                Ok(cd) => Some(cd),
                Err(e) => {
                    error!(?e, "failed to deserialize category data");
                    None
                }
            },
            Err(e) => {
                error!(?e, "failed to read string of category data");
                None
            }
        })
        .flatten()
        .unwrap_or_else(|| {
            let cs = CategorySelection::default_from_pack_core(&core);
            match serde_json::to_string_pretty(&cs) {
                Ok(cs_json) => match dir.write(Self::CATEGORY_SELECTION_FILE_NAME, cs_json) {
                    Ok(_) => {
                        debug!("wrote cat selections to disk after creating a default from pack");
                    }
                    Err(e) => {
                        debug!(?e, "failed to write category data to disk");
                    }
                },
                Err(e) => {
                    error!(?e, "failed to serialize cat selection");
                }
            }
            cs
        });
        Ok(LoadedPack {
            dir,
            core,
            cats_selection,
            dirty: Default::default(),
            current_map_data: Default::default(),
        })
    }
    pub fn tick(
        &mut self,
        etx: &egui::Context,
        _timestamp: f64,
        joko_renderer: &mut joko_render::JokoRenderer,
        link: &Option<Arc<MumbleLink>>,
        default_tex_id: &TextureHandle,
    ) {
        let categories_changed = self.dirty.cats_selection;
        if self.dirty.is_dirty() {
            match self.save() {
                Ok(_) => {}
                Err(e) => {
                    error!(?e, "failed to save marker pack");
                }
            }
        }
        let link = match link {
            Some(link) => link,
            None => return,
        };

        if self.current_map_data.map_id != link.map_id || categories_changed {
            info!(
                self.current_map_data.map_id,
                link.map_id, "current map data is updated."
            );
            self.current_map_data = Default::default();
            if link.map_id == 0 {
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
                        .and_then(|path| self.current_map_data.active_textures.get(path))
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
                        height_offset: marker.props.height_offset.unwrap_or_default(),
                        fade_near: marker.props.fade_near.unwrap_or(-1.0) / INCHES_PER_METER,
                        fade_far: marker.props.fade_far.unwrap_or(-1.0) / INCHES_PER_METER,
                        icon_size: marker.props.icon_size.unwrap_or(1.0),
                    });
                }
            }

            for trail in self
                .core
                .maps
                .get(&link.map_id)
                .unwrap_or(&Default::default())
                .trails
                .iter()
            {
                if let Some(category_attributes) = enabled_cats_list.get(&trail.category) {
                    let mut common_attributes = trail.props.clone();
                    common_attributes.inherit_if_prop_none(category_attributes);
                    if let Some(tex_path) = &common_attributes.texture {
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
                        info!(?trail.props.texture, "no texture attribute on this marker");
                    }
                    let th = common_attributes
                        .texture
                        .as_ref()
                        .and_then(|path| self.current_map_data.active_textures.get(path))
                        .unwrap_or(default_tex_id);
                    let (tex_id, width, height) = match th.id() {
                        egui::TextureId::Managed(tid) => {
                            (tid, th.size()[0] as u16, th.size()[1] as u16)
                        }
                        egui::TextureId::User(_) => unimplemented!(),
                    };
                    let tbin_path = if let Some(tbin) = common_attributes.trail_data_file {
                        tbin
                    } else {
                        info!(?trail, "missing tbin path");
                        continue;
                    };
                    let tbin = if let Some(tbin) = self.core.tbins.get(&tbin_path) {
                        tbin
                    } else {
                        info!(%tbin_path, "failed to find tbin");
                        continue;
                    };
                    if let Some(active_trail) =
                        ActiveTrail::get_vertices_and_texture(&tbin.nodes, width, height, tex_id)
                    {
                        self.current_map_data.active_trails.push(active_trail);
                    }
                }
            }
        }

        for marker in self.current_map_data.active_markers.iter() {
            if let Some(mo) =
                marker.get_vertices_and_texture(link.f_camera_position, link.f_avatar_position)
            {
                joko_renderer.add_billboard(mo);
            }
        }
        for trail in self.current_map_data.active_trails.iter() {
            joko_renderer.add_trail(TrailObject {
                vertices: trail.trail_object.vertices.clone(),
                texture: trail.trail_object.texture,
            });
        }
    }
    pub fn save_all(&mut self) -> Result<()> {
        self.dirty.all = true;
        self.save()
    }
    #[tracing::instrument(skip(self))]
    pub fn save(&mut self) -> Result<()> {
        if std::mem::take(&mut self.dirty.cats_selection) || self.dirty.all {
            match serde_json::to_string_pretty(&self.cats_selection) {
                Ok(cs_json) => match self.dir.write(Self::CATEGORY_SELECTION_FILE_NAME, cs_json) {
                    Ok(_) => {
                        debug!("wrote cat selections to disk after creating a default from pack");
                    }
                    Err(e) => {
                        debug!(?e, "failed to write category data to disk");
                    }
                },
                Err(e) => {
                    error!(?e, "failed to serialize cat selection");
                }
            }
        }
        self.dir
            .create_dir_all(Self::CORE_PACK_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to create xmlpack directory")?;
        let core_dir = self
            .dir
            .open_dir(Self::CORE_PACK_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to open core pack directory")?;
        save_pack_core_to_dir(
            &self.core,
            &core_dir,
            std::mem::take(&mut self.dirty.cats),
            std::mem::take(&mut self.dirty.map_dirty),
            std::mem::take(&mut self.dirty.texture),
            std::mem::take(&mut self.dirty.tbin),
            std::mem::take(&mut self.dirty.all),
        )?;
        Ok(())
    }
}

#[derive(Default)]
pub struct CurrentMapData {
    /// the map to which the current map data belongs to
    pub map_id: u32,
    /// The textures that are being used by the markers, so must be kept alive by this hashmap
    pub active_textures: HashMap<RelativePath, TextureHandle>,
    pub active_markers: Vec<ActiveMarker>,
    pub active_trails: Vec<ActiveTrail>,
}
pub struct ActiveTrail {
    pub trail_object: TrailObject,
}
pub struct ActiveMarker {
    pub pos: glam::Vec3,
    pub width: u16,
    pub height: u16,
    pub texture: u64,
    pub height_offset: f32,
    pub fade_near: f32,
    pub fade_far: f32,
    pub icon_size: f32,
}
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct CategorySelection {
    pub selected: bool,
    pub display_name: String,
    pub children: HashMap<String, CategorySelection>,
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
                common_attributes.inherit_if_prop_none(parent_common_attributes);
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
        egui::ScrollArea::vertical().show(ui, |ui| {
            for cat in selection.values_mut() {
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
        });
    }
}

pub const _BILLBOARD_MAX_VISIBILITY_DISTANCE: f32 = 10000.0;

impl ActiveMarker {
    pub fn get_vertices_and_texture(
        &self,
        cam_pos: Vec3,
        player_pos: Vec3,
    ) -> Option<MarkerObject> {
        let Self {
            pos,
            width,
            height,
            texture,
            height_offset,
            fade_far,
            icon_size,
            ..
        } = *self;
        let distance = pos.distance(player_pos);
        if fade_far > 0.0 && distance > fade_far {
            return None;
        }
        // if marker further than 150 metres, skip rendering them to avoid ugly tiny pixel objects on screen
        if distance > 150.0 {
            return None;
        }
        let mut pos = pos;
        pos.y += height_offset;
        let direction_to_marker = cam_pos - pos;
        let direction_to_side = direction_to_marker.normalize().cross(Vec3::Y);

        // we want to map 100 pixels to one meter in game
        // we are supposed to half the width/height too, as offset from the center will be half of the whole billboard
        // But, i will ignore that as that makes markers too small
        let x_offset = (width as f32 / 100.0) * icon_size;
        let y_offset = (height as f32 / 100.0) * icon_size;
        let bottom_left = MarkerVertex {
            position: (pos - (direction_to_side * x_offset) - (Vec3::Y * y_offset)),
            texture_coordinates: vec2(0.0, 1.0),
            padding: Vec2::default(),
        };

        let top_left = MarkerVertex {
            position: (pos - (direction_to_side * x_offset) + (Vec3::Y * y_offset)),
            texture_coordinates: vec2(0.0, 0.0),
            padding: Vec2::default(),
        };
        let top_right = MarkerVertex {
            position: (pos + (direction_to_side * x_offset) + (Vec3::Y * y_offset)),
            texture_coordinates: vec2(1.0, 0.0),
            padding: Vec2::default(),
        };
        let bottom_right = MarkerVertex {
            position: (pos + (direction_to_side * x_offset) - (Vec3::Y * y_offset)),
            texture_coordinates: vec2(1.0, 1.0),
            padding: Vec2::default(),
        };
        let vertices = [
            top_left,
            bottom_left,
            bottom_right,
            bottom_right,
            top_right,
            top_left,
        ];
        Some(MarkerObject {
            vertices,
            texture,
            distance,
        })
    }
}

impl ActiveTrail {
    pub fn get_vertices_and_texture(
        positions: &[Vec3],
        twidth: u16,
        theight: u16,
        texture: u64,
    ) -> Option<Self> {
        // can't have a trail without atleast two nodes
        if positions.len() < 2 {
            return None;
        }
        let horizontal_offset = twidth as f32 / 200.0; // 100 pixels = 1 meter. divide by another two to get offset from center
        let height = theight as f32 / 50.0; // 50 pixels = 1 meter. just calculate the distance and keeping mod-ing to get the number of times to repeat the texture
        let mut y_offset = 1.0;
        let mut vertices = vec![];
        for two_positions in positions.windows(2) {
            let first = two_positions[0];
            let second = two_positions[1];
            let side = (second - first).normalize().cross(Vec3::Y).normalize() * -1.0;
            let new_offset = (-1.0 * (first.distance(second) / height)) + y_offset;
            let first_left = MarkerVertex {
                position: first - (side * horizontal_offset),
                texture_coordinates: vec2(0.0, y_offset),
                padding: Default::default(),
            };
            let first_right = MarkerVertex {
                position: first + (side * horizontal_offset),
                texture_coordinates: vec2(1.0, y_offset),
                padding: Default::default(),
            };
            let second_left = MarkerVertex {
                position: second - (side * horizontal_offset),
                texture_coordinates: vec2(0.0, new_offset),
                padding: Default::default(),
            };
            let second_right = MarkerVertex {
                position: second + (side * horizontal_offset),
                texture_coordinates: vec2(1.0, new_offset),
                padding: Default::default(),
            };
            y_offset = if new_offset.is_sign_positive() {
                new_offset
            } else {
                1.0 - new_offset.fract().abs()
            };
            vertices.extend([
                second_left,
                first_left,
                first_right,
                first_right,
                second_right,
                second_left,
            ]);
        }

        Some(ActiveTrail {
            trail_object: TrailObject {
                vertices: vertices.into(),
                texture,
            },
        })
    }
}
