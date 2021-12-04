use std::{rc::Rc, sync::mpsc::Receiver};

use anyhow::Context as _;

use glfw::{Glfw, WindowEvent};
use glow::{Context, HasContext};
use jokolink::WindowDimensions;
use log::trace;

use crate::{
    core::window::{OverlayWindow, OverlayWindowConfig},
    gl_error,
};

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

    #[allow(clippy::type_complexity)]
    pub fn create(
        mut config: OverlayWindowConfig,
    ) -> anyhow::Result<(
        OverlayWindow,
        Receiver<(f64, WindowEvent)>,
        Glfw,
        Rc<Context>,
    )> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).context("failed to initialize glfw")?;
        trace!("glfw initialized");

        Self::set_window_hints(&mut glfw, config);

        trace!("set window hints {:?}", &config);

        let (mut window, events) = match glfw.create_window(
            config.framebuffer_width,
            config.framebuffer_height,
            Self::WINDOW_TITLE,
            glfw::WindowMode::Windowed,
        ) {
            Some(w) => w,
            None => anyhow::bail!("failed to create window window"),
        };

        trace!("window created");
        glfw::Context::make_current(&mut window);
        window.set_all_polling(true);
        window.set_store_lock_key_mods(true);

        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
        };
        let gl = Rc::new(gl);

        trace!("got opengl context");
        unsafe {
            gl_error!(gl);
        }
        let passthrough = window.is_mouse_passthrough();
        config.passthrough = passthrough;
        let (x, y) = window.get_pos();
        config.window_pos_x = x;
        config.window_pos_y = y;
        let (width, height) = window.get_framebuffer_size();
        config.framebuffer_height = height as u32;
        config.framebuffer_width = width as u32;
        log::debug!("window created. config is: {:?}", config);
        // WARNING: Need to restart so that egui will get the FrameBuffer size event and it can set the screen_rect property of Rawinput before starting to draw
        // otherwise, it will use the default of (10_000, 10_000) for screen_size. glfw won't bother resizing if we give the same width/height. so, we change them slightly
        window.set_size(width - 1, height - 1);

        Ok((OverlayWindow { window, config }, events, glfw, gl))
    }

    pub fn set_framebuffer_size(&mut self, width: u32, height: u32) {
        if self.config.framebuffer_width != width || self.config.framebuffer_height != height {
            trace!(
                "setting frame buffer size to width: {} and height: {}",
                width,
                height
            );
            self.config.framebuffer_height = height;
            self.config.framebuffer_width = width;
            self.window.set_size(width as i32, height as i32);
        }
    }

    pub fn set_inner_position(&mut self, xpos: i32, ypos: i32) {
        if self.config.window_pos_x != xpos || self.config.window_pos_y != ypos {
            trace!(
                "setting window inner position to x: {} and y: {}",
                xpos,
                ypos
            );
            self.config.window_pos_x = xpos;
            self.config.window_pos_y = ypos;
            self.window.set_pos(xpos, ypos);
        }
    }

    pub fn set_decorations(&mut self, decorated: bool) {
        if self.config.decorated != decorated {
            trace!("setting decorated: {}", decorated);
            self.config.decorated = decorated;
            self.window.set_decorated(decorated);
        }
    }
    pub fn set_passthrough(&mut self, passthrough: bool) {
        if passthrough != self.config.passthrough {
            trace!("setting passthrough: {}", passthrough);
            self.config.passthrough = passthrough;
            self.window.set_mouse_passthrough(passthrough);
        }
    }

    pub fn get_live_inner_size(&mut self) -> (i32, i32) {
        let (width, height) = self.window.get_framebuffer_size();
        if width as u32 != self.config.framebuffer_width {
            log::error!(
                "framebuffer width mismatch with live data. config_width: {}, live_width: {}",
                self.config.framebuffer_width,
                width
            );
            self.config.framebuffer_width = width as u32;
        }
        if height as u32 != self.config.framebuffer_height {
            log::error!(
                "framebuffer height mismatch with live data. config.height: {}, live_height: {}",
                self.config.framebuffer_height,
                height
            );
            self.config.framebuffer_height = height as u32;
        }
        (width, height)
    }

    pub fn get_live_inner_position(&mut self) -> (i32, i32) {
        let (x, y) = self.window.get_pos();
        if x != self.config.window_pos_x {
            log::error!(
                "framebuffer width mismatch with live data. config_width: {}, live_width: {}",
                self.config.window_pos_x,
                x
            );
            self.config.window_pos_x = x;
        }
        if y != self.config.window_pos_y {
            log::error!(
                "framebuffer height mismatch with live data. config.height: {}, live_height: {}",
                self.config.window_pos_y,
                y
            );
            self.config.window_pos_y = y;
        }
        (x, y)
    }

    pub fn get_live_windim(&mut self) -> WindowDimensions {
        self.get_live_inner_position();
        self.get_live_inner_size();
        WindowDimensions {
            x: self.config.window_pos_x,
            y: self.config.window_pos_y,
            width: self.config.framebuffer_width as i32,
            height: (self.config.framebuffer_height as i32),
        }
    }
    pub fn set_always_on_top(&mut self, top: bool) {
        self.window.set_floating(top);
    }
    pub fn swap_buffers(&mut self) {
        use glfw::Context;
        self.window.swap_buffers();
    }
    pub fn set_text_clipboard(&mut self, s: &str) {
        log::debug!("setting clipboard to: {}", s);
        self.window.set_clipboard_string(s);
    }
    pub fn get_text_clipboard(&mut self) -> Option<String> {
        let t = self.window.get_clipboard_string();
        log::debug!("getting clipboard contents. contents: {:?}", &t);
        t
    }
    pub fn should_close(&mut self) -> bool {
        self.window.should_close()
    }
    pub fn attach_to_gw2window(&mut self, new_windim: WindowDimensions) {
        self.set_inner_position(new_windim.x, new_windim.y);
        self.set_framebuffer_size(new_windim.width as u32, new_windim.height as u32);
    }
}
