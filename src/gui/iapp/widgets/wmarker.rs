// use std::{collections::BTreeMap, rc::Rc, sync::Arc};

// use egui::{CtxRef, Window};
// use nalgebra_glm::make_vec3;
// use parking_lot::Mutex;

// use crate::{
//     tactical::{
//         scene::{MarkerNode, SceneNode},
//         xmltypes::{
//             load_markers, xml_category::MarkerCategory, xml_marker::Marker, xml_trail::Trail,
//         },
//     },
//     mlink::MumbleCache,
// };

// pub struct MarkersWindow {
//     pub name: String,
//     pub location: String,
//     pub marker_categories: BTreeMap<String, MarkerCategory>,
//     pub markers: BTreeMap<u32, Vec<Marker>>,
//     pub trails: BTreeMap<u32, Vec<Trail>>,
//     pub marker_scene: SceneNode,
// }

// impl MarkersWindow {
//     pub fn new(gl: Rc<glow::Context>) -> Self {
//         let marker_scene = SceneNode::new(gl);
//         MarkersWindow {
//             name: "Markers".to_string(),
//             location: "./res/tw".to_string(),
//             marker_categories: BTreeMap::new(),
//             markers: BTreeMap::new(),
//             trails: BTreeMap::new(),
//             marker_scene,
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

//                 let mc = mcache.lock();
//                 if let Some(c) = mc.as_ref() {
//                     if let Some(link) = c.link.as_ref() {
//                         if let Some(markers) =
//                             self.markers.get(&link.identity.as_ref().unwrap().map_id)
//                         {
//                             let nodes = markers.iter().map(|v| MarkerNode::from(v)).collect();
//                             // self.marker_scene
//                             //     .vb
//                             //     .update(bytemuck::cast_slice(&markers), glow::DYNAMIC_DRAW);
//                             let vp = nalgebra_glm::look_at_lh(
//                                 &make_vec3(&[
//                                     link.f_camera_position_x,
//                                     link.f_camera_position_y,
//                                     link.f_camera_position_z,
//                                 ]),
//                                 &(make_vec3(&[
//                                     link.f_camera_position_x,
//                                     link.f_camera_position_y,
//                                     link.f_camera_position_z,
//                                 ]) + make_vec3(&[
//                                     link.f_camera_front_x,
//                                     link.f_camera_front_y,
//                                     link.f_camera_front_z,
//                                 ])),
//                                 &make_vec3(&[0.0, 1.0, 0.0]),
//                             );
//                             self.marker_scene.draw(
//                                 Some(&nodes),
//                                 vp,
//                                 make_vec3(&[
//                                     link.f_camera_position_x,
//                                     link.f_camera_position_y,
//                                     link.f_camera_position_z,
//                                 ]),
//                                 make_vec3(&[
//                                     link.f_avatar_position_x,
//                                     link.f_avatar_position_y,
//                                     link.f_avatar_position_z,
//                                 ]),
//                             );
//                         }
//                     }
//                 }
//             }

//             // ui.label(&self.cat_list);
//         });
//     }
// }
