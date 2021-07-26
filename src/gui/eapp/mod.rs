pub mod scene;
pub mod widgets;

use std::{cell::RefCell, rc::Rc};

use egui::{Color32, CtxRef, RawInput, TextureId, Visuals};
use glm::make_vec2;
use glow::{Context, HasContext};
use jokolink::mlink::MumbleLink;

use scene::Painter;
use widgets::MainWindow;

use crate::{gltypes::texture::Texture, window::glfw_window::GlfwWindow};

pub struct EguiApp {
    pub painter: Painter,
    pub main_window: MainWindow,
    pub ctx: CtxRef,
}

impl EguiApp {
    /// Creates a egui CtxRef. ctx has a interior pointer that changes to a new generation of Context everytime we call begin frame.
    /// So, we wrap the CtxRef in Rc so that we all are using the same/latest context inside ctx.
    /// Creates a EguiScene and then uploads the default egui font texture from ctx by caling begin frame.
    pub fn new(gl: Rc<Context>) -> Self {
        let mut ctx = egui::CtxRef::default();

        let mut painter = Painter::new(gl.clone());
        let mut visuals = Visuals::dark();
        visuals.window_shadow.extrusion = 0.0;
        ctx.set_visuals(visuals);
        // upload the main egui font texture
        ctx.begin_frame(RawInput::default());
        let t = ctx.texture();
        let new_texture = Texture::new(gl.clone());
        new_texture.bind();
        let mut pixels = Vec::new();
        for &alpha in &t.pixels {
            let srgba = Color32::from_white_alpha(alpha);
            pixels.push(srgba.r());
            pixels.push(srgba.g());
            pixels.push(srgba.b());
            pixels.push(srgba.a());
        }
        new_texture.update_pixels(&pixels, t.width as u32, t.height as u32);

        painter
            .texture_versions
            .insert(egui::TextureId::Egui, new_texture);
        let _ = ctx.end_frame();
        let main_window = MainWindow::new(gl.clone());
        EguiApp {
            ctx,
            painter,
            main_window,
        }
    }

    /// functions used to upload any textures coming from egui side. assign the texture Id as the User(id) as both of them will be deleted at once when egui calls delete texture
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
    /// this is the primary function that is run in the event loop. we collect the pressed keys/buttons at this moment, get mouse position
    /// and finally, check with the previous values to see if there's any change and upload those events to the raw_input.events vec.
    /// then call begin frameto start uploading the new windows/widgets before calling endframe.
    /// handle any output events, and draw egui.
    pub fn update(
        &mut self,
        overlay_window: &mut GlfwWindow,
        link: &MumbleLink,
        input: &mut RawInput,
    ) -> anyhow::Result<()> {
        let ctx = &mut self.ctx;
        let (width, height) = overlay_window.window_size;
        if ctx.wants_pointer_input() || ctx.wants_keyboard_input() {
            overlay_window.set_passthrough(false);
        } else {
            overlay_window.set_passthrough(true);
        }

        ctx.begin_frame(input.take());

        self.main_window.add_widgets_to_ui(&ctx, link);
        let (egui_output, shapes) = ctx.end_frame();

        if !egui_output.events.is_empty() {
            dbg!(egui_output.events);
        }
        let meshes = ctx.tessellate(shapes);

        self.painter
            .draw_meshes(&meshes, make_vec2(&[width as f32, height as f32]), 0)?;

        Ok(())
    }
}

pub enum UIElements {}
