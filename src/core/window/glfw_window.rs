use std::{rc::Rc, sync::mpsc::Receiver};

use anyhow::Context as _;

use glfw::{Glfw, Window, WindowEvent};
use glow::{Context, HasContext};
use serde::{Deserialize, Serialize};
use x11rb::{
    connection::Connection,
    properties::WmHints,
    protocol::xproto::{get_atom_name, get_property, intern_atom, Atom, AtomEnum},
};

use crate::{gl_error, core::mlink::MumbleManager};

/// Overlay Window Configuration. lightweight and Copy. so, we can pass this around to functions that need the window size/postion
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OverlayWindowConfig {
    /// framebuffer width in pixels
    pub framebuffer_width: u32,
    /// framebuffer height in pixels
    pub framebuffer_height: u32,
    /// can be negative. includes decorations etc.. in screen coordinates
    pub window_pos_x: i32,
    /// can be negative. includes decorations etc.. in screen coordinates
    pub window_pos_y: i32,
    /// whether Window has input passthrough enabled
    pub passthrough: bool,
    /// always on top flag
    pub always_on_top: bool,
    /// transparency flag
    pub transparency: bool,
    /// decorated flag
    pub decorated: bool,
}
impl OverlayWindowConfig {
    pub const WINDOW_HEIGHT: u32 = 600;
    pub const WINDOW_WIDTH: u32 = 800;
    pub const WINDOW_POS_X: i32 = 0;
    pub const WINDOW_POS_Y: i32 = 0;
    pub const PASSTHROUGH: bool = false;
    pub const ALWAYS_ON_TOP: bool = false;
    pub const TRANSPARENCY: bool = false;
    pub const DECORATED: bool = true;
}
impl Default for OverlayWindowConfig {
    fn default() -> Self {
        Self {
            framebuffer_width: Self::WINDOW_WIDTH,
            framebuffer_height: Self::WINDOW_HEIGHT,
            window_pos_x: Self::WINDOW_POS_X,
            window_pos_y: Self::WINDOW_POS_Y,
            passthrough: Self::PASSTHROUGH,
            always_on_top: Self::ALWAYS_ON_TOP,
            transparency: Self::TRANSPARENCY,
            decorated: Self::DECORATED,
        }
    }
}

/// This is the overlay window which wraps the window functions like resizing or getting the present size etc..
/// we will cache a few attributes to avoid calling into system for high frequency variables like
///
pub struct OverlayWindow {
    pub window: Window,
    pub config: OverlayWindowConfig,
}
impl OverlayWindow {
    /// default OpenGL minimum major version
    pub const GL_VERSION_MAJOR: u32 = 4;
    /// default OpenGL minimum minor version
    pub const GL_VERSION_MINOR: u32 = 6;
    /// default window title string
    pub const WINDOW_TITLE: &'static str = "Jokolay";

    /// default MultiSampling samples count
    pub const MULTISAMPLE_COUNT: Option<u32> = None;
}

impl OverlayWindow {
    fn set_window_hints(glfw: &mut Glfw, config: OverlayWindowConfig) {
        glfw.window_hint(glfw::WindowHint::ContextVersion(
            Self::GL_VERSION_MAJOR,
            Self::GL_VERSION_MINOR,
        ));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        glfw.window_hint(glfw::WindowHint::Floating(config.always_on_top));

        glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(
            config.transparency,
        ));

        glfw.window_hint(glfw::WindowHint::MousePassthrough(config.passthrough));

        glfw.window_hint(glfw::WindowHint::Decorated(config.decorated));
        glfw.window_hint(glfw::WindowHint::Samples(Self::MULTISAMPLE_COUNT));
    }
    pub fn create(
        mut config: OverlayWindowConfig,
    ) -> anyhow::Result<(
        OverlayWindow,
        Receiver<(f64, WindowEvent)>,
        Glfw,
        Rc<Context>,
    )> {
        let mut glfw =
            glfw::init(glfw::FAIL_ON_ERRORS).context("failed to initialize glfw window")?;

        Self::set_window_hints(&mut glfw, config);

        // glfw.window_hint(glfw::WindowHint::DoubleBuffer(false));

        let (mut window, events) = glfw
            .create_window(
                config.framebuffer_width,
                config.framebuffer_height,
                Self::WINDOW_TITLE,
                glfw::WindowMode::Windowed,
            )
            .unwrap_or_else(|| {
                log::error!("Failed to create GLFW window");
                panic!()
            });
        glfw::Context::make_current(&mut window);
        window.set_all_polling(true);

        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
        };
        
        
        let passthrough = window.is_mouse_passthrough();
        config.passthrough = passthrough;
        let (x, y) = window.get_pos();
        config.window_pos_x = x;
        config.window_pos_y = y;
        let (width, height) = window.get_framebuffer_size();
        config.framebuffer_height = height as u32;
        config.framebuffer_width = width as u32;
        log::trace!("window created. config is: {:?}", config);
        Ok((OverlayWindow { window, config }, events, glfw, Rc::new(gl)))
    }

    pub fn set_framebuffer_size(&mut self, width: u32, height: u32) {
        if self.config.framebuffer_width != width || self.config.framebuffer_height != height {
            self.config.framebuffer_height = height;
            self.config.framebuffer_width = width;
            self.window.set_size(width as i32, height as i32);
        }
    }

    pub fn set_inner_position(&mut self, xpos: i32, ypos: i32) {
        if self.config.window_pos_x != xpos || self.config.window_pos_y != ypos {
            self.config.window_pos_x = xpos;
            self.config.window_pos_y = ypos;
            self.window.set_pos(xpos, ypos);
        }
    }

    pub fn set_decorations(&mut self, decorated: bool) {
        self.window.set_decorated(decorated);
    }
    pub fn set_passthrough(&mut self, passthrough: bool) {
        if passthrough == self.config.passthrough {
            return;
        }
        self.config.passthrough = passthrough;
        self.window.set_mouse_passthrough(passthrough);
    }

    // pub fn get_inner_size(&mut self) -> (i32, i32) {
    //     self.window.get_framebuffer_size()
    // }

    // pub fn get_inner_position(&mut self) -> (i32, i32) {
    //     self.window.get_pos()
    // }

    pub fn swap_buffers(&mut self) {
        use glfw::Context;
        self.window.swap_buffers();
        // use glow::HasContext;
        // unsafe { self.gl.flush() };
    }

    pub fn should_close(&mut self) -> bool {
        self.window.should_close()
    }
    pub fn attach_to_gw2window(&mut self, mm: &mut MumbleManager) {
        let window_dimensions = mm.get_window_dimensions();
        self.set_inner_position(window_dimensions.x, window_dimensions.y);
        self.set_framebuffer_size(
            window_dimensions.width as u32,
            window_dimensions.height as u32,
        );
    }
}
