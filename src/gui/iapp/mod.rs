use std::rc::Rc;

use glfw::Window;
use glow::Context;
use imgui::{im_str, Condition, Window as IWindow};

use painter::Painter;

pub mod iglfw;
pub mod painter;
pub struct ImguiApp {
    painter: Painter,
    pub ctx: imgui::Context,
}

impl ImguiApp {
    pub fn new(gl: Rc<Context>) -> Self {
        let mut ctx = imgui::Context::create();
        if std::mem::size_of::<imgui::DrawIdx>() != 2 {
            panic!("index not short");
        }

        let painter = Painter::new(
            gl.clone(),
            &mut ctx,
        );
        iglfw::init(&mut ctx);
        ImguiApp {
            ctx,
            painter,
        }
    }

    pub fn update(&mut self, window: &mut Window) -> anyhow::Result<()> {
        let ctx = &mut self.ctx;
        let painter = &mut self.painter;

        let mut show = true;
       
        if ctx.io().want_set_mouse_pos {
            let [x, y] = ctx.io().mouse_pos;
            window.set_cursor_pos(x as _, y as _);
        };
        let ui = ctx.frame();
        ui.show_demo_window(&mut show);
        ui.show_user_guide();
        IWindow::new(im_str!("Hello world"))
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("こんにちは世界！"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.separator();
            });

        iglfw::mouse_cursor_change(&ui, window);
        painter.draw_meshes(ui);
        Ok(())
    }
}
