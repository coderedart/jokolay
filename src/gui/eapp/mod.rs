// pub mod scene;
// pub mod widgets;

// use std::{cell::RefCell, rc::Rc};

// use egui::{Color32, CtxRef, RawInput, TextureId, Visuals};
// use glow::{Context, HasContext};
// use nalgebra_glm::make_vec2;


// use scene::EguiScene;
// use widgets::MainWindow;

// use crate::{gltypes::texture::Texture, window::glfw_window::GlfwWindow};

// pub struct EguiInterface {
//     pub painter: Rc<RefCell<EguiScene>>,
//     pub main_window: Rc<RefCell<MainWindow>>,
//     pub ctx: Rc<RefCell<CtxRef>>,
//     pub overlay_window: Rc<RefCell<GlfwWindow>>,
// }

// impl EguiInterface {
//     /// Creates a egui CtxRef. ctx has a interior pointer that changes to a new generation of Context everytime we call begin frame.
//     /// So, we wrap the CtxRef in Rc so that we all are using the same/latest context inside ctx.
//     /// Creates a EguiScene and then uploads the default egui font texture from ctx by caling begin frame.
//     pub fn new(gl: Rc<Context>, overlay_window: Rc<RefCell<GlfwWindow>>) -> Self {
//         let ctx = Rc::new(RefCell::new(egui::CtxRef::default()));

//         let painter = Rc::new(RefCell::new(EguiScene::new(gl.clone())));
//         let mut visuals = Visuals::dark();
//         visuals.window_shadow.extrusion = 0.0;
//         ctx.borrow_mut().set_visuals(visuals);
//         // upload the main egui font texture
//         ctx.borrow_mut().begin_frame(RawInput::default());
//         let t = ctx.borrow().texture();
//         unsafe {
//             gl.active_texture(glow::TEXTURE0);
//         }
//         let new_texture = Texture::new(gl.clone(), glow::TEXTURE_2D);
//         new_texture.bind();
//         let mut pixels = Vec::new();
//         for &alpha in &t.pixels {
//             let srgba = Color32::from_white_alpha(alpha);
//             pixels.push(srgba.r());
//             pixels.push(srgba.g());
//             pixels.push(srgba.b());
//             pixels.push(srgba.a());
//         }
//         new_texture.update_pixels(&[&pixels], t.width as u32, t.height as u32);

//         painter
//             .borrow_mut()
//             .texture_versions
//             .insert(egui::TextureId::Egui, new_texture);
//         let _ = ctx.borrow_mut().end_frame();
//         let main_window = Rc::new(RefCell::new(MainWindow::new(gl.clone())));
//         EguiInterface {
//             ctx,
//             painter,
//             overlay_window,
//             main_window,
//         }
//     }

//     /// functions used to upload any textures coming from egui side. assign the texture Id as the User(id) as both of them will be deleted at once when egui calls delete texture
//     pub fn upload_user_texture(&self, pixels: &[u8], width: u32, height: u32) -> TextureId {
//         let new_texture = Texture::new(self.painter.borrow().gl.clone(), glow::TEXTURE_2D);
//         new_texture.bind();
//         new_texture.update_pixels(&[pixels], width, height);
//         let tex_id = egui::TextureId::User(new_texture.id.into());
//         self.painter
//             .borrow_mut()
//             .texture_versions
//             .insert(egui::TextureId::User(new_texture.id.into()), new_texture);
//         tex_id
//     }
//     /// this is the primary function that is run in the event loop. we collect the pressed keys/buttons at this moment, get mouse position
//     /// and finally, check with the previous values to see if there's any change and upload those events to the raw_input.events vec.
//     /// then call begin frameto start uploading the new windows/widgets before calling endframe.
//     /// handle any output events, and draw egui.
//     pub fn update(&self, input: RawInput) -> anyhow::Result<()> {
//         let overlay_window = self.overlay_window.clone();
//         let ctx = self.ctx.clone();
//         let painter = self.painter.clone();
//         let (width, height) = overlay_window.borrow().get_inner_size();
//         // let (xpos, ypos) = overlay_window.borrow().get_inner_position();
//         let gl = self.overlay_window.borrow().get_gl_context();
//         // overlay_window.query_input_events(width, height, xpos, ypos);
//         // let (width, height) = overlay_window.window.borrow().get_size();
//         // let width = 800.0_f32;
//         // let height = 600.0_f32;
//         // if !overlay_window.global_input_state.borrow().raw_input.events.is_empty() {
//         // dbg!(&overlay_window.global_input_state.borrow().raw_input.events);
//         // }
//         ctx.borrow_mut().begin_frame(input);
//         // egui::CentralPanel::default().show(&ctx, |ui| {
//         //     ui.add(egui::Label::new("whatever, big text. look at me sempai"));

//         //     if ui.small_button(
//         //         "small button boi"
//         //     ).clicked() {
//         //         println!("small click boi");
//         //     };

//         // });

//         // let mut frame = egui::Frame::default()
//         //     .fill(Color32::BLACK)
//         //     .multiply_with_opacity(0.5);
//         // frame.shadow = Shadow::small_dark();

//         // egui::Window::new("egui window")
//         //     .frame(frame)
//         //     .show(&ctx.borrow(), |ui| {
//         //         if ui.button("click me").clicked() {
//         //             println!("clicked");
//         //         }
//         //     });
//         self.main_window
//             .borrow_mut()
//             .add_widgets_to_ui(&ctx.borrow());
//         let (egui_output, shapes) = ctx.borrow_mut().end_frame();

//         if !egui_output.events.is_empty() {
//             dbg!(egui_output.events);
//         }
//         let meshes = ctx.borrow().tessellate(shapes);

//         painter
//             .borrow_mut()
//             .draw_meshes(&meshes, make_vec2(&[width as f32, height as f32]), 0)?;

//         Ok(())
//     }
// }

// pub enum UIElements {}
