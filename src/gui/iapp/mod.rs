// pub mod iglfw;
// pub mod widgets;
// use std::{cell::RefCell, rc::Rc};

// use crate::window::{glfw_window::GlfwWindow, OverlayWindow};
// use glow::HasContext;
// use iglfw::*;
// use imgui::Context as ICtx;
// use imgui_opengl_renderer::Renderer;

// // use widgets::MainWindow;

// pub struct ImguiInterface {
//     pub painter: Rc<RefCell<Renderer>>,
//     pub main_window: Rc<RefCell<MainWindow>>,
//     pub ctx: Rc<RefCell<ICtx>>,
//     pub overlay_window: Rc<RefCell<GlfwWindow>>,
// }

// impl ImguiInterface {
//     pub fn new(gl: Rc<glow::Context>, overlay_window: Rc<RefCell<GlfwWindow>>) -> Self {
//         let ctx = Rc::new(RefCell::new(ICtx::create()));

//         let painter = Rc::new(RefCell::new(Renderer::new(&mut ctx.borrow_mut(), |s| {
//             overlay_window
//                 .borrow_mut()
//                 .window
//                 .borrow_mut()
//                 .get_proc_address(s) as *const _
//         })));

//         let main_window = Rc::new(RefCell::new(MainWindow::new(gl.clone())));
//         init(&mut ctx.borrow_mut());
//         attach_window(
//             ctx.borrow_mut().io_mut(),
//             &mut overlay_window.borrow_mut().window.borrow_mut(),
//         );
//         ImguiInterface {
//             ctx,
//             painter,
//             overlay_window,
//             main_window,
//         }
//     }

//     /// this is the primary function that is run in the event loop. we collect the pressed keys/buttons at this moment, get mouse position
//     /// and finally, check with the previous values to see if there's any change and upload those events to the raw_input.events vec.
//     /// then call begin frameto start uploading the new windows/widgets before calling endframe.
//     /// handle any output events, and draw egui.
//     pub fn update(&self) -> anyhow::Result<()> {
//         let overlay_window = self.overlay_window.clone();
//         let ctx = self.ctx.clone();
//         let gl = overlay_window.borrow().get_gl_context();
//         let painter = self.painter.clone();
//         let (width, height) = overlay_window.borrow().get_inner_size();

//         unsafe {
//             gl.clear_color(0.0, 0.0, 0.0, 0.0);
//             gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
//         }
//         {
//             let mut ctx = ctx.borrow_mut();
//             overlay_window
//                 .borrow_mut()
//                 .send_events_to_imgui(ctx.io_mut());
//             prepare_frame(
//                 ctx.io_mut(),
//                 &mut overlay_window.borrow_mut().window.borrow_mut(),
//             )
//             .unwrap();

//             let ui = ctx.frame();
//             ui.show_demo_window(&mut true);
//             self.main_window.borrow_mut().add_widgets_to_ui(&ui);
//             prepare_render(&ui, &mut overlay_window.borrow_mut().window.borrow_mut());
//             painter.borrow_mut().render(ui);
//         }

//         Ok(())
//     }
// }

// pub enum UIElements {}
