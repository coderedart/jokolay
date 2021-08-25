use std::{rc::Rc, sync::mpsc::Receiver};

use anyhow::Context as _;

use glfw::{Glfw, Window, WindowEvent};
use glow::Context;

pub struct GlfwWindow {
    pub gl: Rc<glow::Context>,
    pub window: Window,
    pub window_pos: (i32, i32),
    /// size of viewport in pixels
    pub window_size: (i32, i32),
    pub passthrough: bool,
}
impl GlfwWindow {
    pub const INITIAL_WINDOW_WIDTH: u32 = 1920;
    pub const INITIAL_WINDOW_HEIGHT: u32 = 1080;
    pub const GL_VERSION_MAJOR: u32 = 4;
    pub const GL_VERSION_MINOR: u32 = 6;
    pub const WINDOW_TITLE: &'static str = "Jokolay";
    pub fn create(
        passthrough: bool,
    ) -> anyhow::Result<(GlfwWindow, Receiver<(f64, WindowEvent)>, Glfw)> {
        let mut glfw =
            glfw::init(glfw::FAIL_ON_ERRORS).context("failed to initialize glfw window")?;
        glfw.window_hint(glfw::WindowHint::ContextVersion(
            Self::GL_VERSION_MAJOR,
            Self::GL_VERSION_MINOR,
        ));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        glfw.window_hint(glfw::WindowHint::Floating(true));

        glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));

        glfw.window_hint(glfw::WindowHint::MousePassthrough(passthrough));

        glfw.window_hint(glfw::WindowHint::Decorated(false));
        glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

        // glfw.window_hint(glfw::WindowHint::DoubleBuffer(false));

        let (mut window, events) = glfw
            .create_window(
                Self::INITIAL_WINDOW_WIDTH,
                Self::INITIAL_WINDOW_HEIGHT,
                Self::WINDOW_TITLE,
                glfw::WindowMode::Windowed,
            )
            .unwrap_or_else(|| {
                log::error!("Failed to create GLFW window");
                panic!("panicking due to no glfw window")
            });

        glfw::Context::make_current(&mut window);
        window.set_all_polling(true);
        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
        };

        let passthrough = window.is_mouse_passthrough();
        let (xpos, ypos) = window.get_pos();
        let (width, height) = window.get_framebuffer_size();
        Ok((
            GlfwWindow {
                window,
                gl: Rc::new(gl),
                window_pos: (xpos, ypos),
                window_size: (width, height),
                passthrough,
            },
            events,
            glfw,
        ))
    }

    pub fn set_inner_size(&mut self, width: i32, height: i32) {
        if self.window_size.0 != width || self.window_size.1 != height {
            self.window_size = (width, height);
            self.window.set_size(width, height);
        }
    }

    pub fn set_inner_position(&mut self, xpos: i32, ypos: i32) {
        if self.window_pos.0 != xpos || self.window_pos.1 != ypos {
            self.window_pos = (xpos, ypos);
            self.window.set_pos(xpos, ypos);
        }
    }

    pub fn set_decorations(&mut self, decorated: bool) {
        self.window.set_decorated(decorated);
    }
    pub fn set_passthrough(&mut self, passthrough: bool) {
        if passthrough == self.passthrough {
            return;
        }
        self.passthrough = passthrough;
        self.window.set_mouse_passthrough(passthrough);
    }

    // pub fn get_inner_size(&mut self) -> (i32, i32) {
    //     self.window.get_framebuffer_size()
    // }

    // pub fn get_inner_position(&mut self) -> (i32, i32) {
    //     self.window.get_pos()
    // }

    pub fn redraw_request(&mut self) {
        use glfw::Context;
        self.window.swap_buffers();
        // use glow::HasContext;
        // unsafe { self.gl.flush() };
    }

    pub fn should_close(&mut self) -> bool {
        self.window.should_close()
    }

    pub fn get_gl_context(&self) -> Rc<Context> {
        self.gl.clone()
    }
}
