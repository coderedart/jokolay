use std::{collections::BTreeMap, rc::Rc, sync::Arc};

use egui::{CollapsingHeader, CtxRef, Window};
use glow::HasContext;
use nalgebra_glm::Vec3;
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
    pub t: Texture,
    pub draw_markers: bool,
    pub cats_changed: bool,
    pub disabled_cats_window: bool,
}

impl MarkersWindow {
    pub fn new(gl: Rc<glow::Context>) -> Self {
        let t = Texture::new(gl.clone());
        unsafe {
            gl.active_texture(glow::TEXTURE0 + 1);
        }
        t.bind();
        let img = image::open("./res/tex.png").unwrap();
        let img = img.flipv();
        let img = img.into_rgba8();
        let pixels = img.as_ref();
        t.update_pixels(&pixels, img.width(), img.height());
        let marker_manager = MarkerManager::new(gl.clone(), "./res/tw");

        MarkersWindow {
            name: "Markers".to_string(),
            location: "./res/tw".to_string(),
            draw_markers: false,
            t,
            cats_changed: false,
            disabled_cats_window: false,
            marker_manager,
        }
    }
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef, mcache: Arc<Mutex<Option<MumbleManager>>>) {
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
                let mut mc = mcache.lock();
                mc.as_mut().unwrap().try_update();
                if let Some(c) = mc.as_ref() {
                    if let Some(link) = c.link.as_ref() {
                        let map_id = &link.identity.as_ref().unwrap().map_id;
                        if self.cats_changed {
                            self.marker_manager
                                .update_scene_markers_to_current_map(*map_id);
                            self.cats_changed = false;
                        }

                        let camera_position = Vec3::new(
                            link.f_camera_position_x,
                            link.f_camera_position_y,
                            link.f_camera_position_z,
                        );
                        let center = camera_position
                            + Vec3::new(
                                link.f_camera_front_x,
                                link.f_camera_front_y,
                                link.f_camera_front_z,
                            );
                        let view = nalgebra_glm::look_at_lh(
                            &camera_position,
                            &center,
                            &Vec3::new(0.0, 1.0, 0.0),
                        );
                        let projection = nalgebra_glm::perspective_fov_lh(
                            link.identity.as_ref().unwrap().fov,
                            800.0,
                            600.0,
                            0.1,
                            30000.0,
                        );
                        let vp = projection * view;

                        self.marker_manager.scene.draw(vp, camera_position);
                    }
                }
            }
        });
    }

    fn ui_tree_from_marcats(
        cats: &mut BTreeMap<String, MarCat>,
        depth: u32,
        ui: &mut egui::Ui,
        changed: &mut bool,
    ) {
        for c in cats.values_mut() {
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
