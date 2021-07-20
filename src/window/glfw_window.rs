use std::{cell::RefCell, rc::Rc, sync::mpsc::Receiver};

use anyhow::Context as _;

use glfw::{Context as _, Glfw, Window, WindowEvent};
use glow::Context;

pub struct GlfwWindow {
    pub gl: Rc<glow::Context>,
    pub window: Rc<RefCell<Window>>,
    pub window_pos: (i32, i32),
    pub window_size: (i32, i32),
    pub passthrough: bool,
}
impl GlfwWindow {
    pub fn create(
        floating: bool,
        transparent: bool,
        passthrough: bool,
        decorated: bool,
    ) -> anyhow::Result<(GlfwWindow, Receiver<(f64, WindowEvent)>, Glfw)> {
        let mut glfw =
            glfw::init(glfw::FAIL_ON_ERRORS).context("failed to initialize glfw window")?;
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        glfw.window_hint(glfw::WindowHint::Floating(floating));

        glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(transparent));

        glfw.window_hint(glfw::WindowHint::MousePassthrough(passthrough));

        glfw.window_hint(glfw::WindowHint::Decorated(decorated));

        // glfw.window_hint(glfw::WindowHint::DoubleBuffer(false));

        let (mut window, events) = glfw
            .create_window(800, 600, "Jokolay", glfw::WindowMode::Windowed)
            .context("Failed to create GLFW window")?;

        glfw::Context::make_current(&mut window);
        window.set_all_polling(true);
        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
        };
        let passthrough = window.is_mouse_passthrough();
        let window = Rc::new(RefCell::new(window));
        let (xpos, ypos) = window.borrow().get_pos();
        let (width, height) = window.borrow().get_framebuffer_size();
        Ok((
            GlfwWindow {
                window,
                gl: Rc::new(gl),
                window_pos: (xpos, ypos),
                window_size: (width, height),
                passthrough,
            },
            events,
            glfw
        ))
    }

    pub fn set_inner_size(&self, width: i32, height: i32) {
        self.window.borrow_mut().set_size(width, height);
    }

    pub fn set_inner_position(&self, xpos: i32, ypos: i32) {
        self.window.borrow_mut().set_pos(xpos, ypos);
    }

    pub fn set_decorations(&self, decorated: bool) {
        self.window.borrow_mut().set_decorated(decorated);
    }
    pub fn set_passthrough(&mut self, passthrough: bool) {
        if passthrough == self.passthrough {
            return
        }
        self.passthrough = passthrough;
        self.window.borrow_mut().set_mouse_passthrough(passthrough);
        if !passthrough {
            self.window.borrow_mut().focus();
        }
    }
    // pub fn _transparent(&self) {

    // }

    pub fn get_inner_size(&self) -> (i32, i32) {
        self.window.borrow_mut().get_framebuffer_size()
    }

    pub fn get_inner_position(&self) -> (i32, i32) {
        self.window.borrow_mut().get_pos()
    }

    pub fn redraw_request(&self) {
        self.window.borrow_mut().swap_buffers();
        // unsafe { self.gl.flush() };
    }

    pub fn should_close(&self) -> bool {
        self.window.borrow().should_close()
    }

    pub fn get_gl_context(&self) -> Rc<Context> {
        self.gl.clone()
    }
}
