use std::{
    rc::Rc,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use anyhow::Context as _;

use egui::CtxRef;
use glfw::{Glfw, Window, WindowEvent};
use glow::{Context, HasContext};
use jokolink::WindowDimensions;
use serde::{Deserialize, Serialize};

use crate::{core::mlink::MumbleSource, gl_error};

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
    pub const ALWAYS_ON_TOP: bool = true;
    pub const TRANSPARENCY: bool = true;
    pub const DECORATED: bool = false;
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
    pub gw2_last_checked: Instant,
    #[cfg(target_os = "linux")]
    pub platform_data: super::linux::LinuxPlatformData,
    #[cfg(target_os = "windows")]
    pub platform_data: super::windows::WindowsPlatformData,
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
        mumble_src: &mut MumbleSource,
    ) -> anyhow::Result<(
        OverlayWindow,
        Receiver<(f64, WindowEvent)>,
        Glfw,
        Rc<Context>,
    )> {
        let gw2_last_checked = Instant::now();
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
        let gl = Rc::new(gl);
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
        log::trace!("window created. config is: {:?}", config);
        #[cfg(target_os = "linux")]
        let platform_data = super::linux::LinuxPlatformData::new(&window, mumble_src);
        #[cfg(target_os = "windows")]
        let platform_data = super::windows::WindowsPlatformData::new(&window, mumble_src);

        Ok((
            OverlayWindow {
                window,
                config,
                gw2_last_checked,
                platform_data,
            },
            events,
            glfw,
            gl,
        ))
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
        if self.config.decorated != decorated {
            self.config.decorated = decorated;
            self.window.set_decorated(decorated);
        }
    }
    pub fn set_passthrough(&mut self, passthrough: bool) {
        if passthrough != self.config.passthrough {
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

    pub fn swap_buffers(&mut self) {
        use glfw::Context;
        self.window.swap_buffers();
        // use glow::HasContext;
        // unsafe { self.gl.flush() };
    }

    pub fn should_close(&mut self) -> bool {
        self.window.should_close()
    }
    pub fn attach_to_gw2window(&mut self, new_windim: WindowDimensions) {
        self.set_inner_position(new_windim.x, new_windim.y);
        self.set_framebuffer_size(new_windim.width as u32, new_windim.height as u32);
    }

    pub fn tick(&mut self, ctx: &CtxRef) {
        if self.gw2_last_checked.elapsed() > Duration::from_secs(2) {
            self.gw2_last_checked = Instant::now();
            if self.is_gw2_alive() {
                let gw2_windim = self.get_gw2_windim();
                let ow_windim: WindowDimensions = self.get_live_windim();
                if gw2_windim != ow_windim {
                    log::info!(
                        "resizing to match gw2. old dimensions: {:#?}, new dimensions: {:#?}",
                        ow_windim,
                        gw2_windim
                    );
                    self.attach_to_gw2window(gw2_windim);
                }
            } else {
                log::debug!("gw2 process is not alive anymore");
                self.window.set_should_close(true);
            }
        }
        if ctx.wants_pointer_input() || ctx.wants_keyboard_input() {
            self.set_passthrough(false);
        } else {
            self.set_passthrough(true);
        }
    }
}
