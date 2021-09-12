// use egui::{CollapsingHeader, CtxRef, Response, Widget, Window};

// use jokolink::mlink::MumbleLink;

// use crate::tactical::localtypes::{
//     category::CatSelectionTree, category::IMCategory, manager::MarkerManager,
// };

// impl MarkerManager {
//     pub fn tick(&mut self, ctx: CtxRef, link: &MumbleLink, show_marker_window: &mut bool) {
//         if self.state.active_cats_changed || self.state.current_map != link.context.map_id {
//             self.active_markers.clear();
//             self.active_trails.clear();
//             for (pack_index, pack) in self.packs.iter_mut().enumerate() {
//                 pack.update_active_markers(
//                     link.identity.map_id,
//                     pack_index,
//                     &mut self.active_markers,
//                 );
//                 pack.update_active_trails(
//                     link.identity.map_id,
//                     pack_index,
//                     &mut self.active_trails,
//                 );
//             }
//             self.state.active_cats_changed = false;
//             self.state.current_map = link.context.map_id;
//         }
//         Window::new("Marker Manager")
//             .open(show_marker_window)
//             .show(&ctx, |ui| {
//                 self.ui(ui);
//             });
//         let mut show_cstree = self.state.show_cat_selection_window;
//         Window::new("category selection")
//             .open(&mut show_cstree)
//             .scroll(true)
//             .default_height(300.0)
//             .default_width(400.0)
//             .show(&ctx, |ui| {
//                 for (_pack_index, pack) in self.packs.iter_mut().enumerate() {
//                     let mut active_cats_changed = self.state.active_cats_changed;
//                     CollapsingHeader::new(pack.path.0).show(ui, |ui| {
//                         // let mut changed = false;
//                         if let Some(ref mut cstree) = pack.cat_selection_tree {
//                             cstree.build_cat_selection_ui(
//                                 ui,
//                                 &mut active_cats_changed,
//                                 &pack.global_cats,
//                             );
//                         }
//                     });
//                     self.state.active_cats_changed = active_cats_changed;
//                 }
//             });
//         self.state.show_cat_selection_window = show_cstree;
//     }
// }
// impl Widget for &mut MarkerManager {
//     fn ui(self, ui: &mut egui::Ui) -> Response {
//         ui.text_edit_singleline(&mut self.state.load_folder_path);
//         if ui.button("load markers").clicked() {
//             unimplemented!();
//             // *self = MarkerManager::new(&PathBuf::from_str(&self.state.load_folder_path).unwrap());
//         }

//         ui.checkbox(&mut self.draw_markers, "draw markers");
//         ui.checkbox(
//             &mut self.state.show_cat_selection_window,
//             "category selection Window",
//         )
//         // ui.checkbox(
//         //     &mut self.show_active_current_map_markers_window,
//         //     "show active markers",
//         // )
//     }
// }

// impl CatSelectionTree {
//     pub fn build_cat_selection_ui(
//         &mut self,
//         ui: &mut egui::Ui,
//         changed: &mut bool,
//         global_cats: &Vec<IMCategory>,
//     ) {
//         ui.horizontal(|ui| {
//             if ui.checkbox(&mut self.enabled, "").changed() {
//                 *changed = true;
//             }
//             CollapsingHeader::new(&global_cats[self.category_index.0].cat.display_name)
//                 .default_open(false)
//                 .enabled(!self.children.is_empty())
//                 .id_source(self.state.id)
//                 .show(ui, |ui| {
//                     for child in &mut self.children {
//                         child.build_cat_selection_ui(ui, changed, global_cats);
//                     }
//                 });
//         });
//     }
// }
