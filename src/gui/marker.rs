use std::{collections::HashSet, path::PathBuf, rc::Rc};

use egui::{Checkbox, CollapsingHeader, CtxRef, Response, Widget, Window};

use jokolink::mlink::MumbleLink;
use uuid::Uuid;

use crate::tactical::localtypes::{
    manager::MarkerManager, CatSelectionTree, IMCategory, MarkerPack,
};

impl MarkerManager {
    pub fn tick(&mut self, ctx: CtxRef, link: &MumbleLink) {
        Window::new("Marker Manager").show(&ctx, |ui| {
            self.ui(ui);
        });
        if self.state.show_cat_selection_window {
            
            Window::new("category selection")
                .scroll(true)
                .default_height(300.0)
                .default_width(400.0)
                .show(&ctx, |ui| {
                    for ( pack_index, pack) in self.packs.iter_mut().enumerate() {
                        let active_markers = &mut self.active_markers;
                        CollapsingHeader::new(pack.path.to_str().unwrap())
                        .show(ui, |ui| {
                            let mut changed = false;
                            if let Some(ref mut cstree) = pack.cat_selection_tree {
                                cstree.build_cat_selection_ui(
                                    ui,
                                    &mut changed,
                                    &pack.global_cats,
                                );
                            }
                            if changed {
                                pack
                                    .fill_muuid_cindex_map(link.identity.map_id, pack_index,active_markers);
                            }
                        });
                       
                    }
                   
                });
        }
         
       
    }
}
impl Widget for &mut MarkerManager {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        ui.text_edit_singleline(&mut self.state.load_folder_path);
        if ui.button("load markers").clicked() {
            self.location = PathBuf::from(&self.state.load_folder_path);
            *self = MarkerManager::new(&self.location);
        }

        ui.checkbox(&mut self.draw_markers, "draw markers");
        ui.checkbox(&mut self.state.show_cat_selection_window, "category selection Window")
        // ui.checkbox(
        //     &mut self.show_active_current_map_markers_window,
        //     "show active markers",
        // )

       
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
