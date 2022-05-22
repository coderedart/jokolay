use bevy_math::{dvec2, DVec2, IVec2};
use bevy_utils::HashMap;
use bevy_window::{Window, WindowDescriptor, WindowId, WindowMode};

use glfw::Glfw;
use raw_window_handle::HasRawWindowHandle;
use std::sync::mpsc::Receiver;

/// This is a Non Send struct because a lot of windowing functions MUST be called from the Main
/// Thread.
pub struct GlfwBackend {
    pub(crate) windows: HashMap<WindowId, WindowState>,
    pub(crate) glfw_context: Glfw,
    // Marker to make this !Send. negative Marker Trait types are not stable yet, so we have to
    // use this PhantomData for now.
    _not_send_sync: core::marker::PhantomData<*const ()>,
}

pub struct WindowState {
    pub(crate) window: glfw::Window,
    pub(crate) events_receiver: Receiver<(f64, glfw::WindowEvent)>,
    pub(crate) passthrough: bool,
    pub(crate) dimensions: DVec2,
    pub(crate) cursor_position: DVec2,
}
impl WindowState {
    /// tells us whether we need to push a update position
    ///
    /// if window is not passthrough, no need to update cursor positon manually as
    /// the normal event loop will take care of that. false
    ///
    /// if cursor position is same as previous cursor position, no need to check, as there's no point
    /// in updating. false
    ///
    /// if window area doesn't contain cursor position and because window is passthrough, so, egui is
    /// not using cursor position for drags or such, we don't need to create an event for cursor position
    ///
    /// finally, if window is passthorugh, cursor position changed since last update and it is within
    /// the window area bounds, we need to send a cursor update event.
    pub(crate) fn update_cursor_position(&mut self) -> Option<DVec2> {
        if !self.passthrough {
            return None;
        }
        let new_cursor_position = self.window.get_cursor_pos().into();
        if new_cursor_position == self.cursor_position {
            return None;
        }
        let _old_cursor_position = self.cursor_position;
        self.cursor_position = new_cursor_position;
        if Self::window_contains_position(self.dimensions, new_cursor_position) {
            Some(new_cursor_position)
        } else {
            None
        }
    }
    pub fn set_passthrough(&mut self, passthrough: bool) {
        if self.passthrough != passthrough {
            self.window.set_mouse_passthrough(passthrough);
            self.passthrough = self.window.is_mouse_passthrough();
        }
    }
    /// checks whether cursor position lies within the window rectangle area
    /// x and y must be greater than or equal to 0.0, which is probably the top left corner and start
    /// of rectangle. and must be less than the width and height of the window as that's the
    /// far corner of the rectangle.
    fn window_contains_position(dimensions: DVec2, position: DVec2) -> bool {
        position.x >= 0.0
            && position.y >= 0.0
            && position.x <= dimensions.x
            && position.y <= dimensions.y
    }
}
impl GlfwBackend {
    pub(crate) fn new() -> Self {
        let glfw_context = glfw::init(glfw::LOG_ERRORS).expect("failed to initialize Glfw");
        Self {
            windows: Default::default(),
            glfw_context,
            _not_send_sync: Default::default(),
        }
    }
    pub fn create_window(
        &mut self,
        window_id: WindowId,
        window_descriptor: &WindowDescriptor,
    ) -> Window {
        // our global hints irrespective of provided options

        // vulkan backend needs this hint
        self.glfw_context
            .window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        // scale window sizes based on the monitor
        // self.glfw_context
        //     .window_hint(glfw::WindowHint::ScaleToMonitor(true));
        // set always on top hint
        self.glfw_context
            .window_hint(glfw::WindowHint::Floating(true));
        // set mouse passthrough hint
        self.glfw_context
            .window_hint(glfw::WindowHint::MousePassthrough(true));
        // set sticky keys so that keys get "released" events when we lose focus / become passthrough

        // hints based on window descriptor

        // set transparency. only at startup, can't change at runtime
        self.glfw_context
            .window_hint(glfw::WindowHint::TransparentFramebuffer(
                window_descriptor.transparent,
            ));
        // set resizeable
        self.glfw_context
            .window_hint(glfw::WindowHint::Resizable(window_descriptor.resizable));
        // set decorations at startup.
        self.glfw_context
            .window_hint(glfw::WindowHint::Decorated(window_descriptor.decorations));

        // create window. panics if window cannot be created with the provided settings.
        let (mut window, events_receiver): (glfw::Window, _) =
            create_window(&mut self.glfw_context, window_descriptor);

        // enable all events polling
        window.set_all_polling(true);
        //
        // window.set_sticky_keys(true);
        // window.set_sticky_mouse_buttons(true);
        // TODO: size constraints part of window_descriptor. I'm too lazy for that.

        // lets check that transparency worked. this is important for Jokolay :)
        if window_descriptor.transparent {
            assert!(window.is_framebuffer_transparent());
        }
        // check passthrough
        assert!(window.is_mouse_passthrough());
        // check always on top
        assert!(window.is_floating());
        // I have no idea what these settings are for, so i will avoid these.
        // if window_descriptor.cursor_locked {
        //     window.set_cursor_mode(glfw::CursorMode::Disabled);
        // }
        //
        // if window_descriptor.cursor_visible {
        //     window.set_cursor_mode(glfw::CursorMode::Normal);
        // }

        // we cannot decide window position with hints, so we set it now.
        if let Some(position) = window_descriptor.position {
            window.set_pos(position.x as i32, position.y as i32);
        }
        // lets collect the needed things for bevy.
        let position = window.get_pos();
        let position = Some(IVec2::new(position.0, position.1));
        let inner_size = window.get_size();
        let scale_factor = window.get_content_scale().0 as f64;
        let raw_window_handle = window.raw_window_handle();
        let passthrough = window.is_mouse_passthrough();
        let cursor_position = window.get_cursor_pos();
        self.windows.insert(
            window_id,
            WindowState {
                window,
                events_receiver,
                passthrough,
                dimensions: dvec2(inner_size.0 as f64, inner_size.1 as f64),
                cursor_position: cursor_position.into(),
            },
        );
        Window::new(
            window_id,
            window_descriptor,
            inner_size.0 as u32,
            inner_size.1 as u32,
            scale_factor,
            position,
            raw_window_handle,
        )
    }

    pub fn get_window(&self, id: WindowId) -> Option<&WindowState> {
        self.windows.get(&id)
    }
    pub fn get_window_mut(&mut self, id: &WindowId) -> Option<&mut WindowState> {
        self.windows.get_mut(id)
    }
}
/// Creates a `glfw::Window` based on the options of `WindowDescriptor` like Fullscreen or
/// Windowed etc.. this will also used the width, height too. Primarily just to gather this
/// huge ugly mess into a single function to not affect the readability of the rest of the codebase.
fn create_window(
    glfw_context: &mut Glfw,
    window_descriptor: &WindowDescriptor,
) -> (glfw::Window, Receiver<(f64, glfw::WindowEvent)>) {
    match window_descriptor.mode {
        WindowMode::BorderlessFullscreen => {
            // just get primary monitor's current video mode and use its current settings to make a
            // full screen window.
            glfw_context.with_primary_monitor(|glfw_context, mon| {
                let m = mon.expect("failed to get primary monitor");
                let (width, height) = m
                    .get_video_mode()
                    .map(|video_mode| {
                        glfw_context
                            .window_hint(glfw::WindowHint::RedBits(Some(video_mode.red_bits)));
                        glfw_context
                            .window_hint(glfw::WindowHint::GreenBits(Some(video_mode.green_bits)));
                        glfw_context
                            .window_hint(glfw::WindowHint::BlueBits(Some(video_mode.blue_bits)));
                        glfw_context.window_hint(glfw::WindowHint::RefreshRate(Some(
                            video_mode.refresh_rate,
                        )));
                        (video_mode.width, video_mode.height)
                    })
                    .unwrap_or_else(|| {
                        let (_, _, w, h) = m.get_workarea();
                        let (sx, sy) = m.get_content_scale();
                        assert!(w > 0, "monitor width less than zero {w}");
                        assert!(h > 0, "monitor height less than zero {h}");
                        (w as u32 * sx as u32, h as u32 * sy as u32)
                    });

                glfw_context.create_window(
                    width,
                    height,
                    &window_descriptor.title,
                    glfw::WindowMode::FullScreen(m),
                )
            })
        }
        WindowMode::Fullscreen => glfw_context.with_primary_monitor(|glfw_context, mon| {
            // get the optimal video mode of the primary monitor and use that for a fullscreen window
            let mon = mon.expect("failed to get primary monitor");
            let video_mode = get_best_video_mode(mon);

            glfw_context.window_hint(glfw::WindowHint::RedBits(Some(video_mode.red_bits)));
            glfw_context.window_hint(glfw::WindowHint::GreenBits(Some(video_mode.green_bits)));
            glfw_context.window_hint(glfw::WindowHint::BlueBits(Some(video_mode.blue_bits)));
            glfw_context.window_hint(glfw::WindowHint::RefreshRate(Some(video_mode.refresh_rate)));
            glfw_context.create_window(
                video_mode.width,
                video_mode.height,
                &window_descriptor.title,
                glfw::WindowMode::FullScreen(mon),
            )
        }),
        WindowMode::SizedFullscreen => {
            // get the closest matching video mode of the monitor compared to the
            // provided settings in window_descriptor and use that to make a fullscreen window
            glfw_context.with_primary_monitor(|glfw_context, mon| {
                let mon = mon.expect("failed to get primary monitor");
                let video_mode = get_fitting_video_mode(
                    mon,
                    window_descriptor.width as u32,
                    window_descriptor.height as u32,
                );
                glfw_context.window_hint(glfw::WindowHint::RedBits(Some(video_mode.red_bits)));
                glfw_context.window_hint(glfw::WindowHint::GreenBits(Some(video_mode.green_bits)));
                glfw_context.window_hint(glfw::WindowHint::BlueBits(Some(video_mode.blue_bits)));
                glfw_context
                    .window_hint(glfw::WindowHint::RefreshRate(Some(video_mode.refresh_rate)));
                glfw_context.create_window(
                    video_mode.width,
                    video_mode.height,
                    &window_descriptor.title,
                    glfw::WindowMode::FullScreen(mon),
                )
            })
        }
        _ => {
            // a normal window
            let WindowDescriptor {
                width,
                height,
                position,
                scale_factor_override,
                ..
            } = window_descriptor;
            let (width, height) = (
                *width as u32 * scale_factor_override.unwrap_or(1.0) as u32,
                *height as u32 * scale_factor_override.unwrap_or(1.0) as u32,
            );

            glfw_context
                .create_window(
                    width,
                    height,
                    &window_descriptor.title,
                    glfw::WindowMode::Windowed,
                )
                .map(|(mut window, events)| {
                    if let Some(position) = position {
                        window.set_pos(
                            position[0] as i32 * scale_factor_override.unwrap_or(1.0) as i32,
                            position[1] as i32 * scale_factor_override.unwrap_or(1.0) as i32,
                        );
                    }
                    (window, events)
                })
        }
    }
    .expect("failed to create window")
}
/// This function gets the video modes from a monitor. then, sorts the video modes by using steps:
/// 1. calculates absolute_difference_1 between the provided width and video_mode_width_1.
/// 2. calculates absolute_difference_2 between the provided width and video_mode_width_2.
/// 3. returns the comparison of absolute_difference_1 and 2, making the "closest" match of given
/// width and the video_mode widths as "less". so, closest match will get sorted to the start of vec.
/// if they are both equal, it compares the height absolute difference and then refresh rate.
/// finally, returns the first video mode in the list of video modes
pub fn get_fitting_video_mode(monitor: &glfw::Monitor, width: u32, height: u32) -> glfw::VidMode {
    let mut modes = monitor.get_video_modes();

    fn abs_diff(a: u32, b: u32) -> u32 {
        if a > b {
            return a - b;
        }
        b - a
    }

    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match abs_diff(a.width, width).cmp(&abs_diff(b.width, width)) {
            Equal => match abs_diff(a.height, height).cmp(&abs_diff(b.height, height)) {
                Equal => b.refresh_rate.cmp(&a.refresh_rate),
                default => default,
            },
            default => default,
        }
    });

    *modes.first().unwrap()
}

/// we just get the list of video modes, compare them by width, height and refresh rates.
/// sort them. get the largest (last) video mode
pub fn get_best_video_mode(monitor: &glfw::Monitor) -> glfw::VidMode {
    let mut modes = monitor.get_video_modes();
    modes.sort_by_cached_key(|vid_mode| (vid_mode.width, vid_mode.height, vid_mode.refresh_rate));
    *modes.last().unwrap()
}
