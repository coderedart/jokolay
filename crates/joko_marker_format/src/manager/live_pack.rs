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
use uuid::Uuid;

use crate::{
    io::{load_pack_core_from_dir, save_pack_core_to_dir},
    pack::{Category, CommonAttributes, PackCore, RelativePath},
    INCHES_PER_METER,
};
use jokolink::MumbleLink;
use miette::{bail, Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

pub(crate) struct LoadedPack {
    /// The directory inside which the pack data is stored
    /// There should be a subdirectory called `core` which stores the pack core
    /// Files related to Jokolay thought will have to be stored directly inside this directory, to keep the xml subdirectory clean.
    /// eg: Active categories, activation data etc..
    pub dir: Arc<Dir>,
    /// The actual xml pack.
    pub core: PackCore,
    /// The selection of categories which are "enabled" and markers belonging to these may be rendered
    cats_selection: HashMap<String, CategorySelection>,
    dirty: Dirty,
    activation_data: ActivationData,
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
/// This is the activation data per pack
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActivationData {
    /// this is for markers which are global and only activate once regardless of account
    pub global: IndexMap<Uuid, ActivationType>,
    /// this is the activation data per character
    /// for markers which trigger once per character
    pub character: IndexMap<String, IndexMap<Uuid, ActivationType>>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ActivationType {
    /// clean these up when the map is changed
    ReappearOnMapChange,
    /// clean these up when the timestamp is reached
    TimeStamp(time::OffsetDateTime),
    Instance(std::net::IpAddr),
}
impl LoadedPack {
    const CORE_PACK_DIR_NAME: &str = "core";
    const CATEGORY_SELECTION_FILE_NAME: &str = "cats.json";
    const ACTIVATION_DATA_FILE_NAME: &str = "activation.json";

    pub fn new(core: PackCore, dir: Arc<Dir>) -> Self {
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
            activation_data: Default::default(),
        }
    }
    pub fn category_sub_menu(&mut self, ui: &mut egui::Ui) {
        CategorySelection::recursive_selection_ui(
            &mut self.cats_selection,
            ui,
            &mut self.dirty.cats_selection,
        );
    }
    pub fn load_from_dir(dir: Arc<Dir>) -> Result<Self> {
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

        let cats_selection = (if dir.exists(Self::ACTIVATION_DATA_FILE_NAME) {
            match dir.read_to_string(Self::CATEGORY_SELECTION_FILE_NAME) {
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
            }
        } else {
            None
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
        let activation_data = (if dir.exists(Self::ACTIVATION_DATA_FILE_NAME) {
            match dir.read_to_string(Self::ACTIVATION_DATA_FILE_NAME) {
                Ok(contents) => match serde_json::from_str(&contents) {
                    Ok(cd) => Some(cd),
                    Err(e) => {
                        error!(?e, "failed to deserialize activation data");
                        None
                    }
                },
                Err(e) => {
                    error!(?e, "failed to read string of category data");
                    None
                }
            }
        } else {
            None
        })
        .flatten()
        .unwrap_or_default();
        Ok(LoadedPack {
            dir,
            core,
            cats_selection,
            dirty: Default::default(),
            current_map_data: Default::default(),
            activation_data,
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
            self.on_map_changed(etx, link, default_tex_id);
        }
          let z_near = joko_renderer.get_z_near();
        for marker in self.current_map_data.active_markers.values() {
            if let Some(mo) = marker.get_vertices_and_texture(link, z_near) {
                joko_renderer.add_billboard(mo);
            }
        }
        for trail in self.current_map_data.active_trails.values() {
            joko_renderer.add_trail(TrailObject {
                vertices: trail.trail_object.vertices.clone(),
                texture: trail.trail_object.texture,
            });
        }
    }
    fn on_map_changed(
        &mut self,
        etx: &egui::Context,
        link: &MumbleLink,
        default_tex_id: &TextureHandle,
    ) {
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
        for (index, marker) in self
            .core
            .maps
            .get(&link.map_id)
            .unwrap_or(&Default::default())
            .markers
            .iter()
            .enumerate()
        {
            if let Some(category_attributes) = enabled_cats_list.get(&marker.category) {
                let mut attrs = marker.attrs.clone();
                attrs.inherit_if_attr_none(category_attributes);
                let key = &marker.guid;
                if let Some(behavior) = attrs.get_behavior() {
                    use crate::pack::Behavior;
                    if match behavior {
                        Behavior::AlwaysVisible => false,
                        Behavior::ReappearOnMapChange
                        | Behavior::ReappearOnDailyReset
                        | Behavior::OnlyVisibleBeforeActivation
                        | Behavior::ReappearAfterTimer
                        | Behavior::ReappearOnMapReset
                        | Behavior::WeeklyReset => self.activation_data.global.contains_key(key),
                        Behavior::OncePerInstance => self
                            .activation_data
                            .global
                            .get(key)
                            .map(|a| match a {
                                ActivationType::Instance(a) => a == &link.server_address,
                                _ => false,
                            })
                            .unwrap_or_default(),
                        Behavior::DailyPerChar => self
                            .activation_data
                            .character
                            .get(&link.name)
                            .map(|a| a.contains_key(key))
                            .unwrap_or_default(),
                        Behavior::OncePerInstancePerChar => self
                            .activation_data
                            .character
                            .get(&link.name)
                            .map(|a| {
                                a.get(key)
                                    .map(|a| match a {
                                        ActivationType::Instance(a) => a == &link.server_address,
                                        _ => false,
                                    })
                                    .unwrap_or_default()
                            })
                            .unwrap_or_default(),
                        Behavior::WvWObjective => {
                            false // ???
                        }
                    } {
                        continue;
                    }
                }
                if let Some(tex_path) = attrs.get_icon_file() {
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
                    info!("no texture attribute on this marker");
                }
                let th = attrs
                    .get_icon_file()
                    .and_then(|path| self.current_map_data.active_textures.get(path))
                    .unwrap_or(default_tex_id);
                let texture_id = match th.id() {
                    egui::TextureId::Managed(i) => i,
                    egui::TextureId::User(_) => todo!(),
                };

                let max_pixel_size = attrs.get_max_size().copied().unwrap_or(2048.0); // default taco max size
                let min_pixel_size = attrs.get_min_size().copied().unwrap_or(5.0); // default taco min size
                self.current_map_data.active_markers.insert(
                    index,
                    ActiveMarker {
                        texture_id,
                        _texture: th.clone(),
                        attrs,
                        pos: marker.position,
                        max_pixel_size,
                        min_pixel_size,
                    },
                );
            }
        }

        for (index, trail) in self
            .core
            .maps
            .get(&link.map_id)
            .unwrap_or(&Default::default())
            .trails
            .iter()
            .enumerate()
        {
            if let Some(category_attributes) = enabled_cats_list.get(&trail.category) {
                let mut common_attributes = trail.props.clone();
                common_attributes.inherit_if_attr_none(category_attributes);
                if let Some(tex_path) = common_attributes.get_texture() {
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
                    info!("no texture attribute on this marker");
                }
                let th = common_attributes
                    .get_texture()
                    .and_then(|path| self.current_map_data.active_textures.get(path))
                    .unwrap_or(default_tex_id);

                let tbin_path = if let Some(tbin) = common_attributes.get_trail_data() {
                    tbin
                } else {
                    info!(?trail, "missing tbin path");
                    continue;
                };
                let tbin = if let Some(tbin) = self.core.tbins.get(tbin_path) {
                    tbin
                } else {
                    info!(%tbin_path, "failed to find tbin");
                    continue;
                };
                if let Some(active_trail) = ActiveTrail::get_vertices_and_texture(
                    &common_attributes,
                    &tbin.nodes,
                    th.clone(),
                ) {
                    self.current_map_data
                        .active_trails
                        .insert(index, active_trail);
                }
            }
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
pub(crate) struct CurrentMapData {
    /// the map to which the current map data belongs to
    pub map_id: u32,
    /// The textures that are being used by the markers, so must be kept alive by this hashmap
    pub active_textures: HashMap<RelativePath, TextureHandle>,
    /// The key is the index of the marker in the map markers
    /// Their position in the map markers serves as their "id" as uuids can be duplicates.
    pub active_markers: IndexMap<usize, ActiveMarker>,
    /// The key is the position/index of this trail in the map trails. same as markers
    pub active_trails: IndexMap<usize, ActiveTrail>,
}

/*
- activation data with uuids and track the latest timestamp that will be activated
- category activation data -> track and changes to propagate to markers of this map
- current active markers, which will keep track of their original marker, so as to propagate any changes easily
*/
pub struct ActiveTrail {
    pub trail_object: TrailObject,
    pub texture_handle: TextureHandle,
}
/// This is an active marker.
/// It stores all the info that we need to scan every frame
pub(crate) struct ActiveMarker {
    /// texture id from managed textures
    pub texture_id: u64,
    /// owned texture handle to keep it alive
    pub _texture: TextureHandle,
    /// position
    pub pos: Vec3,
    /// billboard must not be bigger than this size in pixels
    pub max_pixel_size: f32,
    /// billboard must not be smaller than this size in pixels
    pub min_pixel_size: f32,
    pub attrs: CommonAttributes,
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
                common_attributes.inherit_if_attr_none(parent_common_attributes);
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
    pub fn get_vertices_and_texture(&self, link: &MumbleLink, z_near: f32) -> Option<MarkerObject> {
        let Self {
            texture_id,
            pos,
            attrs,
            _texture,
            max_pixel_size,
            min_pixel_size,
            ..
        } = self;
        // let width = *width;
        // let height = *height;
        let texture_id = *texture_id;
        let pos = *pos;
        // filters
        if let Some(mounts) = attrs.get_mount() {
            if let Some(current) = link.mount {
                if !mounts.contains(current) {
                    return None;
                }
            } else {
                return None;
            }
        }
        let height_offset = attrs.get_height_offset().copied().unwrap_or(1.5); // default taco height offset
        let fade_near = attrs.get_fade_near().copied().unwrap_or(-1.0) / INCHES_PER_METER;
        let fade_far = attrs.get_fade_far().copied().unwrap_or(-1.0) / INCHES_PER_METER;
        let icon_size = attrs.get_icon_size().copied().unwrap_or(1.0);
        let player_distance = pos.distance(link.player_pos);
        let camera_distance = pos.distance(link.cam_pos);
        let fade_near_far = Vec2::new(fade_near, fade_far);

        let alpha = attrs.get_alpha().copied().unwrap_or(1.0);
        let color = attrs.get_color().copied().unwrap_or_default();
        /*
           1. we need to filter the markers
               1. statically - mapid, character, map_type, race, profession
               2. dynamically - achievement, behavior, mount, fade_far, cull
               3. force hide/show by user discretion
           2. for active markers (not forcibly shown), we must do the dynamic checks every frame like behavior
           3. store the state for these markers activation data, and temporary data like bounce
        */
        /*
        skip if:
        alpha is 0.0
        achievement id/bit is done (maybe this should be at map filter level?)
        behavior (activation)
        cull
        distance > fade_far
        visibility (ingame/map/minimap)
        mount
        specialization
        */
        if fade_far > 0.0 && player_distance > fade_far {
            return None;
        }
        // markers are 1 meter in width/height by default
        let mut pos = pos;
        pos.y += height_offset;
        let direction_to_marker = link.cam_pos - pos;
        let direction_to_side = direction_to_marker.normalize().cross(Vec3::Y);

        let far_offset = {
            let dpi = if link.dpi_scaling <= 0 {
                96.0
            } else {
                link.dpi as f32
            } / 96.0;
            let gw2_width = link.client_size.as_vec2().x / dpi;

            // offset (half width i.e. distance from center of the marker to the side of the marker)
            const SIDE_OFFSET_FAR: f32 = 1.0;
            // the size of the projected on to the near plane
            let near_offset = SIDE_OFFSET_FAR * icon_size * (z_near / camera_distance);
            // convert the near_plane width offset into pixels by multiplying the near_ffset with gw2 window width
            let near_offset_in_pixels = near_offset * gw2_width;

            // we will clamp the texture width between min and max widths, and make sure that it is less than gw2 window width
            let near_offset_in_pixels = near_offset_in_pixels
                .clamp(*min_pixel_size, *max_pixel_size)
                .min(gw2_width / 2.0);

            let near_offset_of_marker = near_offset_in_pixels / gw2_width;
            near_offset_of_marker * camera_distance / z_near
        };
        // let pixel_ratio = width as f32 * (distance / z_near);// (near width / far width) = near_z / far_z;
        // we want to map 100 pixels to one meter in game
        // we are supposed to half the width/height too, as offset from the center will be half of the whole billboard
        // But, i will ignore that as that makes markers too small
        let x_offset = far_offset;
        let y_offset = x_offset; // seems all markers are squares
        let bottom_left = MarkerVertex {
            position: (pos - (direction_to_side * x_offset) - (Vec3::Y * y_offset)),
            texture_coordinates: vec2(0.0, 1.0),
            alpha,
            color,
            fade_near_far,
        };

        let top_left = MarkerVertex {
            position: (pos - (direction_to_side * x_offset) + (Vec3::Y * y_offset)),
            texture_coordinates: vec2(0.0, 0.0),
            alpha,
            color,
            fade_near_far,
        };
        let top_right = MarkerVertex {
            position: (pos + (direction_to_side * x_offset) + (Vec3::Y * y_offset)),
            texture_coordinates: vec2(1.0, 0.0),
            alpha,
            color,
            fade_near_far,
        };
        let bottom_right = MarkerVertex {
            position: (pos + (direction_to_side * x_offset) - (Vec3::Y * y_offset)),
            texture_coordinates: vec2(1.0, 1.0),
            alpha,
            color,
            fade_near_far,
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
            texture: texture_id,
            distance: player_distance,
        })
    }
}

impl ActiveTrail {
    fn get_vertices_and_texture(
        attrs: &CommonAttributes,
        positions: &[Vec3],
        texture: TextureHandle,
    ) -> Option<Self> {
        // can't have a trail without atleast two nodes
        if positions.len() < 2 {
            return None;
        }
        let alpha = attrs.get_alpha().copied().unwrap_or(1.0);
        let fade_near = attrs.get_fade_near().copied().unwrap_or(-1.0) / INCHES_PER_METER;
        let fade_far = attrs.get_fade_far().copied().unwrap_or(-1.0) / INCHES_PER_METER;
        let fade_near_far = Vec2::new(fade_near, fade_far);
        let color = attrs.get_color().copied().unwrap_or([0u8; 4]);
        // default taco width
        let horizontal_offset = 20.0 / INCHES_PER_METER;
        // scale it trail scale
        let horizontal_offset = horizontal_offset * attrs.get_trail_scale().copied().unwrap_or(1.0);
        let height = horizontal_offset * 2.0;
        let mut y_offset = 1.0;
        let mut vertices = vec![];
        for two_positions in positions.windows(2) {
            let first = two_positions[0];
            let second = two_positions[1];
            // right side of the vector from first to second
            let right_side = (second - first).normalize().cross(Vec3::Y).normalize();

            let new_offset = (-1.0 * (first.distance(second) / height)) + y_offset;
            let first_left = MarkerVertex {
                position: first - (right_side * horizontal_offset),
                texture_coordinates: vec2(0.0, y_offset),
                alpha,
                color,
                fade_near_far,
            };
            let first_right = MarkerVertex {
                position: first + (right_side * horizontal_offset),
                texture_coordinates: vec2(1.0, y_offset),
                alpha,
                color,
                fade_near_far,
            };
            let second_left = MarkerVertex {
                position: second - (right_side * horizontal_offset),
                texture_coordinates: vec2(0.0, new_offset),
                alpha,
                color,
                fade_near_far,
            };
            let second_right = MarkerVertex {
                position: second + (right_side * horizontal_offset),
                texture_coordinates: vec2(1.0, new_offset),
                alpha,
                color,
                fade_near_far,
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
                texture: match texture.id() {
                    egui::TextureId::Managed(i) => i,
                    egui::TextureId::User(_) => todo!(),
                },
            },
            texture_handle: texture,
        })
    }
}
