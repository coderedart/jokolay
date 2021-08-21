pub mod marker;

use egui::{ClippedMesh, Widget, Window};

use crate::JokolayApp;

impl JokolayApp {
    pub fn tick(&mut self) -> Vec<ClippedMesh> {
        let (_width, _height) = self.overlay_window.window_size;
        if self.ctx.wants_pointer_input() || self.ctx.wants_keyboard_input() {
            self.overlay_window.set_passthrough(false);
        } else {
            self.overlay_window.set_passthrough(true);
        }

        self.ctx.begin_frame(self.state.input.take());
        let ctx = self.ctx.clone();
        Window::new("Jokolay").show(&ctx, |ui| {
            self.ui(ui);
        });
        if self.state.show_mumble_window {
            Window::new("Mumble Info").scroll(true).show(&ctx, |ui| {
                ui.label(format!("{:#?}", self.mumble_manager.link));
            });
        };
        if self.state.show_marker_manager {
            self.marker_manager
                .tick(ctx.clone(), &self.mumble_manager.link);
        };
        let (egui_output, shapes) = ctx.end_frame();

        if !egui_output.events.is_empty() {
            dbg!(egui_output.events);
        }
        ctx.tessellate(shapes)
    }
}
impl Widget for &mut JokolayApp {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.checkbox(&mut self.state.show_mumble_window, "show Mumble Setup");
        ui.checkbox(&mut self.state.show_marker_manager, "show Marker Manager")
    }
}

// functions used to upload any textures coming from egui side. assign the texture Id as the User(id) as both of them will be deleted at once when egui calls delete texture
// pub fn upload_user_texture(&mut self, pixels: &[u8], width: u32, height: u32) -> TextureId {
//     let new_texture = Texture::new(self.painter.gl.clone());
//     new_texture.bind();
//     new_texture.update_pixels(pixels, width, height);
//     let tex_id = egui::TextureId::User(new_tex);
//     self.painter
//         .texture_versions
//         .insert(egui::TextureId::User(new_texture.id.into()), new_texture);
//     tex_id
// }
