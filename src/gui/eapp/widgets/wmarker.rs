use std::{collections::HashSet, rc::Rc, time::Instant};

use egui::{Checkbox, CollapsingHeader, CtxRef, Response, Widget, Window};
use glm::make_vec3;

use jokolink::mlink::MumbleLink;

use crate::tactical::{
    xmltypes::{xml_category::OverlayData, CatTree, MarkerPack},
    MarkerManager,
};

pub struct MarkersWindow {
    pub name: String,
    pub location: String,
    pub marker_manager: MarkerManager,
    pub draw_markers: bool,
    pub cats_changed: bool,
    pub disabled_cats_window: bool,
}

impl MarkersWindow {
    pub fn new(_gl: Rc<glow::Context>) -> Self {
        let marker_manager = MarkerManager::new("./res");

        MarkersWindow {
            name: "Markers".to_string(),
            location: "./res/tw".to_string(),
            draw_markers: false,
            cats_changed: false,
            disabled_cats_window: false,
            marker_manager,
        }
    }
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef, link: &MumbleLink) {
        Window::new(&self.name).show(&ctx, |ui| {
            ui.text_edit_singleline(&mut self.location);
            if ui.button("load markers").clicked() {
                self.marker_manager = MarkerManager::new(&self.location);
            }

            ui.checkbox(&mut self.disabled_cats_window, "show category tree");
            ui.checkbox(&mut self.draw_markers, "draw markers");

            if self.disabled_cats_window {
                Window::new("category selection")
                    .scroll(true)
                    .default_height(300.0)
                    .default_width(400.0)
                    .show(&ctx, |ui| {
                        let mut changed = self.cats_changed;
                        for pack in &mut self.marker_manager.marker_packs {
                            ui.collapsing(&pack.cat_display.template.display_name.clone(), |ui| {
                                pack.cat_display.build_cat_selection_ui(ui, &mut changed);
                            });
                        }
                        self.cats_changed = changed;
                    });
            }

            if self.draw_markers {
                // let active_cats = HashSet::new();
                let map_id = link.identity.map_id;
                let mut present_markers = vec![];

                for pack in self.marker_manager.marker_packs.iter_mut() {
                    present_markers.append(&mut pack.get_present_map_markers_with_inherit(map_id));
                }

                let camera_position = glm::make_vec3(&link.f_camera_position);
                let center = camera_position + make_vec3(&link.f_camera_front);
                let view = glm::look_at_lh(&camera_position, &center, &glm::vec3(0.0, 1.0, 0.0));
                let projection =
                    glm::perspective_fov_lh(link.identity.fov, 800.0, 600.0, 0.1, 30000.0);
                let _vp = projection * view;

                // self.marker_manager.scene.draw(vp, camera_position);
            }
        });
    }
}

impl CatTree {
    pub fn build_cat_selection_ui(&mut self, ui: &mut egui::Ui, changed: &mut bool) {
        ui.horizontal(|ui| {
            let mut checked = self.enabled.load(std::sync::atomic::Ordering::Relaxed);
            if ui.checkbox(&mut checked, "").changed() {
                *changed = true;
            }
            self.enabled
                .store(checked, std::sync::atomic::Ordering::Relaxed);
            CollapsingHeader::new(&self.template.display_name)
                .default_open(false)
                .enabled(!self.children.is_empty())
                .id_source(self.id)
                .show(ui, |ui| {
                    for child in &mut self.children {
                        child.build_cat_selection_ui(ui, changed);
                    }
                });
        });
    }
}
