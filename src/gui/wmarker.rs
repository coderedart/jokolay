use std::{collections::HashSet, rc::Rc, time::Instant};

use egui::{Checkbox, CollapsingHeader, CtxRef, Response, Widget, Window};
use glm::make_vec3;

use jokolink::mlink::MumbleLink;
use uuid::Uuid;

use crate::tactical::localtypes::{CatSelectionTree, IMCategory, MarkerPack};

pub struct MarkersWindow {
    pub name: String,
    pub location: String,
    pub marker_manager: MarkerPack,
    pub draw_markers: bool,
    pub cats_changed: bool,
    pub show_cat_selection_window: bool,
    pub show_active_current_map_markers_window: bool,
    pub active_markers: HashSet<(Uuid, usize)>,
}

impl MarkersWindow {
    pub fn new(_gl: Rc<glow::Context>) -> Self {
        let marker_manager = MarkerPack::new("./res/tw".into());

        MarkersWindow {
            name: "Markers".to_string(),
            location: "./res/tw".to_string(),
            draw_markers: false,
            cats_changed: false,
            show_cat_selection_window: false,
            marker_manager,
            show_active_current_map_markers_window: false,
            active_markers: HashSet::new(),
        }
    }
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef, link: &MumbleLink) {
        Window::new(&self.name).show(&ctx, |ui| {
            ui.text_edit_singleline(&mut self.location);
            if ui.button("load markers").clicked() {
                self.marker_manager = MarkerPack::new((&self.location).into());
            }

            ui.checkbox(&mut self.show_cat_selection_window, "show category tree");
            ui.checkbox(&mut self.draw_markers, "draw markers");
            ui.checkbox(
                &mut self.show_active_current_map_markers_window,
                "show active markers",
            );
            if self.show_cat_selection_window {
                Window::new("category selection")
                    .scroll(true)
                    .default_height(300.0)
                    .default_width(400.0)
                    .show(&ctx, |ui| {
                        let mut changed = self.cats_changed;
                        if let Some(ref mut cstree) = self.marker_manager.cat_selection_tree {
                            cstree.build_cat_selection_ui(
                                ui,
                                &mut changed,
                                &self.marker_manager.global_cats,
                            );
                        }
                        self.cats_changed = changed;
                    });
            }
            if self.cats_changed {
                self.marker_manager
                    .fill_muuid_cindex_map(link.identity.map_id, &mut self.active_markers);
                    self.cats_changed = false;
            }
            
            // if self.draw_markers {
            //     // let active_cats = HashSet::new();
            //     let map_id = link.identity.map_id;
            //     let mut present_markers = vec![];

            //     for pack in self.marker_manager.marker_packs.iter_mut() {
            //         present_markers.append(&mut pack.get_present_map_markers_with_inherit(map_id));
            //     }

            //     let camera_position = glm::make_vec3(&link.f_camera_position);
            //     let center = camera_position + make_vec3(&link.f_camera_front);
            //     let view = glm::look_at_lh(&camera_position, &center, &glm::vec3(0.0, 1.0, 0.0));
            //     let projection =
            //         glm::perspective_fov_lh(link.identity.fov, 800.0, 600.0, 0.1, 30000.0);
            //     let _vp = projection * view;

            //     // self.marker_manager.scene.draw(vp, camera_position);
            // }
        });
    }
}

impl CatSelectionTree {
    pub fn build_cat_selection_ui(
        &mut self,
        ui: &mut egui::Ui,
        changed: &mut bool,
        global_cats: &Vec<IMCategory>,
    ) {
        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.enabled, "").changed() {
                *changed = true;
            }
            CollapsingHeader::new(&global_cats[self.category_index].cat.display_name)
                .default_open(false)
                .enabled(!self.children.is_empty())
                .id_source(self.id)
                .show(ui, |ui| {
                    for child in &mut self.children {
                        child.build_cat_selection_ui(ui, changed, global_cats);
                    }
                });
        });
    }
}
