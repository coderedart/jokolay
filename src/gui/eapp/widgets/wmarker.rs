use std::{collections::BTreeMap, rc::Rc, sync::Arc};

use egui::{CollapsingHeader, CtxRef, Window};
use glm::make_vec3;
use glow::HasContext;
use jokolink::mlink::MumbleLink;
use parking_lot::Mutex;

use crate::{
    gltypes::texture::Texture,
    mlink::MumbleManager,
    tactical::{xmltypes::MarCat, MarkerManager},
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
    pub fn new(gl: Rc<glow::Context>) -> Self {
      
        let marker_manager = MarkerManager::new(gl.clone(), "./res/tw");

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
                self.marker_manager.load_markers(&self.location);
            }

            ui.checkbox(&mut self.disabled_cats_window, "show category tree");
            ui.checkbox(&mut self.draw_markers, "draw markers");

            if self.disabled_cats_window {
                Window::new("category selection")
                    .scroll(true)
                    .default_height(600.0)
                    .show(&ctx, |ui| {
                        Self::ui_tree_from_marcats(
                            &mut self.marker_manager.mar_cats,
                            0,
                            ui,
                            &mut self.cats_changed,
                        );
                    });
            }

            if self.draw_markers {
                let map_id = link.identity.map_id;
                if self.cats_changed {
                    self.marker_manager
                        .update_scene_markers_to_current_map(map_id);
                    self.cats_changed = false;
                }

                let camera_position = glm::make_vec3(&link.f_camera_position);
                let center = camera_position + make_vec3(&link.f_camera_front);
                let view = glm::look_at_lh(&camera_position, &center, &glm::vec3(0.0, 1.0, 0.0));
                let projection =
                    glm::perspective_fov_lh(link.identity.fov, 800.0, 600.0, 0.1, 30000.0);
                let vp = projection * view;

                self.marker_manager.scene.draw(vp, camera_position);
            }
        });
    }

    fn ui_tree_from_marcats(
        cats: &mut Vec<MarCat>,
        depth: u32,
        ui: &mut egui::Ui,
        changed: &mut bool,
    ) {
        for c in cats.iter_mut() {
            if CollapsingHeader::new(&c.xml_cat.display_name)
                .id_source(c.id)
                .default_open(c.enabled)
                .show(ui, |ui| {
                    Self::ui_tree_from_marcats(&mut c.children, depth + 1, ui, changed);
                })
                .header_response
                .clicked()
            {
                c.enabled = !c.enabled;
                *changed = true;
            };
        }
    }
}
