use std::{cell::RefCell, rc::Rc};

use egui::{Color32, CtxRef, RawInput, TextureId, any, epaint::Shadow};
use glow::Context;
use nalgebra_glm::make_vec2;

use crate::{glc::renderer::texture::Texture, window::OverlayWindow};

use super::scene::EguiScene;

pub struct EguiApp {
    pub ctx: CtxRef,
    pub painter: Rc<RefCell<EguiScene>>,
    pub overlay_window: Rc<OverlayWindow>,
}

impl EguiApp {
    pub fn new(gl: Rc<Context>, overlay_window: Rc<OverlayWindow>) -> Self {
        let mut ctx = egui::CtxRef::default();
        let painter = Rc::new(RefCell::new(EguiScene::new(gl.clone())));
        // upload the main egui font texture
        ctx.begin_frame(RawInput::default());
        let t = ctx.texture();
        let new_texture = Texture::new(gl.clone(), glow::TEXTURE_2D);
        new_texture.bind();
        let mut pixels = Vec::new();
        for &alpha in &t.pixels {
            let srgba = Color32::from_white_alpha(alpha);
            pixels.push(srgba.r());
            pixels.push(srgba.g());
            pixels.push(srgba.b());
            pixels.push(srgba.a());
        }
        new_texture.update_pixels(&[&pixels], t.width as u32, t.height as u32);

        painter
            .borrow_mut()
            .texture_versions
            .insert(egui::TextureId::Egui, new_texture);
        let _ = ctx.end_frame();

        EguiApp {
            ctx,
            painter,
            overlay_window,
        }
    }

    pub fn upload_user_texture(&self, pixels: &[u8], width: u32, height: u32) -> TextureId {
        let new_texture = Texture::new(self.painter.borrow().gl.clone(), glow::TEXTURE_2D);
        new_texture.bind();
        new_texture.update_pixels(&[pixels], width, height);
        let tex_id = egui::TextureId::User(new_texture.id.into());
        self.painter
            .borrow_mut()
            .texture_versions
            .insert(egui::TextureId::User(new_texture.id.into()), new_texture);
        tex_id
    }

    pub fn update(&self) -> anyhow::Result<()> {
        let overlay_window = self.overlay_window.clone();
        let mut ctx = self.ctx.clone();
        let painter = self.painter.clone();

        overlay_window.query_input_events();
        let (width, height) = overlay_window.window.borrow().get_size();

        ctx.begin_frame(
            overlay_window
                .global_input_state
                .borrow_mut()
                .raw_input
                .take(),
        );
        // egui::CentralPanel::default().show(&ctx, |ui| {
        //     ui.add(egui::Label::new("whatever, big text. look at me sempai"));

        //     if ui.small_button(
        //         "small button boi"
        //     ).clicked() {
        //         println!("small click boi");
        //     };

        // });
   
        egui::Window::new("egui window")
            .show(&ctx, |ui| {
                ui.add(egui::Label::new(
                    "label inside window. please look at me sempai",
                ));
                if ui.button("click me").clicked() {
                    println!("clicked");
                }
            });
        // egui::SidePanel::left("best panel ever").show(&ctx.borrow(), |ui| {
        //     ui.add(egui::Label::new("ffs. what's with the blur"));
        // });
        let (_, shapes) = ctx.end_frame();

        let meshes = ctx.tessellate(shapes);

        painter
            .borrow_mut()
            .draw_meshes(&meshes, make_vec2(&[width as f32, height as f32]), 0)?;
            Ok(())
    }
}
