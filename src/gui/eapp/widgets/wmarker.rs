// use std::{collections::{BTreeMap, BTreeSet}, rc::Rc, sync::Arc};

// use egui::{CtxRef, Window};
// use glow::HasContext;
// use nalgebra_glm::Vec3;
// use parking_lot::Mutex;

// use crate::{mlink::MumbleCache, tactical::xmltypes::MarCat};

// pub struct MarkersWindow {
//     pub name: String,
//     pub location: String,
//     pub marker_categories: Vec<MarCat>,
//     pub marker_scene: SceneNode,
//     pub t: Texture,
//     pub draw_markers: bool,
//     pub disabled_cats_changed: bool,
//     pub disabled_cats_window: bool,
// }

// impl MarkersWindow {
//     pub fn new(gl: Rc<glow::Context>) -> Self {
//         let marker_scene = SceneNode::new(gl.clone());
//         let t = Texture::new(gl.clone(), glow::TEXTURE_2D);
//         unsafe {
//             gl.active_texture(glow::TEXTURE0 + 1);
//         }
//         t.bind();
//         let img = image::open("./res/tex.png").unwrap();
//         let img = img.flipv();
//         let img = img.into_rgba8();
//         let pixels = img.as_ref();
//         t.update_pixels(&[pixels], img.width(), img.height());

//         MarkersWindow {
//             name: "Markers".to_string(),
//             location: "./res/tw".to_string(),
//             marker_categories: BTreeMap::new(),
//             markers: BTreeMap::new(),
//             trails: BTreeMap::new(),
//             marker_scene,
//             draw_markers: false,
//             t,
//             disabled_cats: BTreeSet::new(),
//             disabled_cats_changed: false,
//             disabled_cats_window: false,
//         }
//     }
//     pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef, mcache: Arc<Mutex<Option<MumbleCache>>>) {
//         Window::new(&self.name).show(&ctx, |ui| {
//             ui.text_edit_singleline(&mut self.location);
//             if ui.button("load markers").clicked() {
//                 let (marker_categories, markers, trails) = load_markers(&self.location);
//                 self.marker_categories = marker_categories;
//                 self.markers = markers;
//                 self.trails = trails;
//             }

//             ui.label(format!(
//                 "categories: {}, markers: {}, trails: {}",
//                 self.marker_categories.len(),
//                 self.markers.len(),
//                 self.trails.len()
//             ));
//             ui.checkbox(&mut self.disabled_cats_window, "show category tree");
//             ui.checkbox(&mut self.draw_markers, "draw markers");

//             if self.disabled_cats_window {
//                 Window::new("category selection").show(&ctx, |ui|{
                    
//                 });
//             }

//             if self.draw_markers {
//                 let mut mc = mcache.lock();
//                 mc.as_mut().unwrap().update_link().unwrap();
//                 if let Some(c) = mc.as_ref() {
//                     if let Some(link) = c.link.as_ref() {
//                         if let Some(markers) =
//                             self.markers.get(&link.identity.as_ref().unwrap().map_id)
//                         {
//                             let nodes = markers.iter().map(|v| MarkerNode::from(v)).collect();
//                             // self.marker_scene
//                             //     .vb
//                             //     .update(bytemuck::cast_slice(&markers), glow::DYNAMIC_DRAW);
//                             let camera_position = Vec3::new(
//                                 link.f_camera_position_x,
//                                 link.f_camera_position_y,
//                                 link.f_camera_position_z,
//                             );
//                             let center = camera_position
//                                 + Vec3::new(
//                                     link.f_camera_front_x,
//                                     link.f_camera_front_y,
//                                     link.f_camera_front_z,
//                                 );
//                             let view = nalgebra_glm::look_at_lh(
//                                 &camera_position,
//                                 &center,
//                                 &Vec3::new(0.0, 1.0, 0.0),
//                             );
//                             let projection = nalgebra_glm::perspective_fov_lh(
//                                 link.identity.as_ref().unwrap().fov,
//                                 800.0,
//                                 600.0,
//                                 0.1,
//                                 30000.0,
//                             );
//                             let vp = projection * view;
//                             unsafe {
//                                 self.marker_scene.gl.clear_color(0.0, 0.0, 0.0, 0.0);
//                                 self.marker_scene.gl.clear(
//                                     glow::COLOR_BUFFER_BIT
//                                         | glow::DEPTH_BUFFER_BIT
//                                         | glow::STENCIL_BUFFER_BIT,
//                                 );

//                                 self.marker_scene.gl.active_texture(glow::TEXTURE0);
//                                 self.t.bind();
//                                 self.marker_scene.gl.clear_color(0.0, 0.0, 0.0, 0.0);
//                                 self.marker_scene.gl.clear(
//                                     glow::COLOR_BUFFER_BIT
//                                         | glow::DEPTH_BUFFER_BIT
//                                         | glow::STENCIL_BUFFER_BIT,
//                                 );
//                             }
//                             self.marker_scene.draw(
//                                 Some(&nodes),
//                                 vp,
//                                 camera_position,
//                                 Vec3::new(
//                                     link.f_avatar_position_x,
//                                     link.f_avatar_position_y,
//                                     link.f_avatar_position_z,
//                                 ),
//                             );
//                         }
//                     }
//                 }
//             }
//         });
//     }
// }

