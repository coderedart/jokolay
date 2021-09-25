pub mod marker;

use egui::{ClippedMesh, Widget, Window};

use crate::JokolayApp;

impl JokolayApp {
    pub fn tick(&mut self) -> Vec<ClippedMesh> {
        self.core.tick(&self.ctx);

        let input = self
            .core
            .im
            .process_events(&mut self.core.ow, self.core.rr.egui_gl.gl.clone());

        self.ctx.begin_frame(input);

        let ctx = self.ctx.clone();
        Window::new("J").scroll(true).show(&ctx, |ui| {
            self.state.ui(ui);
        });

        let mut show_mumble = self.state.show_mumble_window;
        Window::new("Mumble Info")
            .open(&mut show_mumble)
            .scroll(true)
            .show(&ctx, |ui| {
                ui.label(format!("{:#?}", &self.core.mbm.link));
            });
        self.state.show_mumble_window = show_mumble;

        // self.marker_manager.tick(
        //     ctx.clone(),
        //     &self.mumble_manager.link,
        //     &mut self.state.show_marker_manager,
        // );
        let (egui_output, shapes) = ctx.end_frame();

        if !egui_output.events.is_empty() {
            dbg!(egui_output.events);
        }
        ctx.tessellate(shapes)
    }
}

impl Widget for &mut crate::EState {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.checkbox(&mut self.show_mumble_window, "Mumble Live");
        ui.checkbox(&mut self.show_marker_manager, "show Marker Manager")
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
