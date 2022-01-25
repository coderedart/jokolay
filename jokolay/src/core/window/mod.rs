use circular_queue::CircularQueue;
use flume::Sender;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tap::{Conv, Pipe};
use time::OffsetDateTime;

use egui::{Event, Key, PointerButton, RawInput};

use glfw::{Action, Glfw, WindowEvent};
use glm::{I32Vec2, U16Vec2, U32Vec2, Vec2};

use anyhow::Context as _;

use jokolink::WindowDimensions;
use tracing::{error, info, trace};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// This is the overlay window which wraps the window functions like resizing or getting the present size etc..
/// we will cache a few attributes to avoid calling into system for high frequency variables like
#[derive(Debug)]
pub struct OverlayWindow {
    pub window: glfw::Window,
    pub glfw: Glfw,
    _rdev_thread: (std::thread::JoinHandle<()>, Arc<AtomicBool>),
    rdev_events: flume::Receiver<rdev::Event>,
    pub events: Receiver<(f64, WindowEvent)>,
    pub window_state: WindowState,
}

#[derive(Debug, Clone)]
pub struct WindowState {
    pub size: U32Vec2,
    pub position: I32Vec2,
    // pub transient_for: Option<usize>,
    pub framebuffer_size: U32Vec2,
    pub scale: Vec2,
    pub latest_local_events: CircularQueue<WindowEvent>,
    pub latest_global_events: CircularQueue<rdev::Event>,
    pub cursor_position: Vec2,
    pub present_time: time::OffsetDateTime,
    pub glfw_time: f64,
    pub average_frame_rate: u16,
    pub previous_fps_reset: f64,
    pub current_frame_number: u16,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            size: Default::default(),
            position: Default::default(),
            framebuffer_size: Default::default(),
            scale: Default::default(),
            latest_local_events: CircularQueue::with_capacity(10),
            latest_global_events: CircularQueue::with_capacity(10),
            cursor_position: Default::default(),
            present_time: OffsetDateTime::now_utc(),
            glfw_time: Default::default(),
            average_frame_rate: Default::default(),
            previous_fps_reset: Default::default(),
            current_frame_number: Default::default(),
        }
    }
}

impl WindowState {
    pub fn i32_to_u32(size: (i32, i32)) -> anyhow::Result<U32Vec2> {
        Ok(U32Vec2::new(
            size.0
                .try_into()
                .with_context(|| format!("size returned negative values. size: {:#?}", size))?,
            size.1.try_into().with_context(|| {
                format!(
                    "framebuffer size returned negative values. frame buffer size: {:#?}",
                    size
                )
            })?,
        ))
    }
}

#[derive(Debug, Clone)]
pub enum WindowCommand {
    Resize(u32, u32),
    Repos(i32, i32),
    Transparent(bool),
    Passthrough(bool),
    Decorated(bool),
    AlwaysOnTop(bool),
    ShouldClose(bool),
    SwapInterval(glfw::SwapInterval),
    SetTransientFor(u32),
    SetClipBoard(String),
    GetClipBoard(Sender<Option<String>>),
    ApplyConfig,
}

impl OverlayWindow {
    /// default window title string
    pub const WINDOW_TITLE: &'static str = "Jokolay";

    #[allow(clippy::type_complexity)]
    #[tracing::instrument]
    pub fn create(framebuffer_size: U16Vec2) -> anyhow::Result<OverlayWindow> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).context("failed to initialize glfw")?;
        trace!("glfw initialized");
        Self::set_window_hints(&mut glfw);

        trace!("creating window");

        let (mut window, events) = match glfw.create_window(
            framebuffer_size[0] as u32,
            framebuffer_size[1] as u32,
            Self::WINDOW_TITLE,
            glfw::WindowMode::Windowed,
        ) {
            Some(w) => w,
            None => anyhow::bail!("failed to create window window"),
        };

        trace!("window created");
        window.set_all_polling(true);
        window.set_store_lock_key_mods(true);
        let position = window.get_pos().pipe(|(x, y)| I32Vec2::new(x, y));
        let framebuffer_size = window
            .get_framebuffer_size()
            .pipe(WindowState::i32_to_u32)?;
        let size = window.get_size().pipe(WindowState::i32_to_u32)?;
        let transparent = window.is_framebuffer_transparent();
        let decorations = window.is_decorated();
        info!("transparent: {}, decorations: {}", transparent, decorations);
        let scale = window.get_content_scale().pipe(|s| Vec2::new(s.0, s.1));
        let cursor_position = window
            .get_cursor_pos()
            .pipe(|cp| Vec2::new(cp.0 as f32, cp.1 as f32));
        let glfw_time = glfw.get_time();
        let (sender, receiver) = flume::bounded(500);
        let rdev_thread_signaller = Arc::new(AtomicBool::new(false));
        let rdev_thread_signaller_copy = rdev_thread_signaller.clone();
        let rdev_thread_handle = std::thread::spawn(move || {
            if let Err(e) = rdev::listen(move |ev| {
                let _ = sender.send(ev);
                if rdev_thread_signaller_copy.load(std::sync::atomic::Ordering::Relaxed) {
                    panic!("rdev_thread_signaller is true")
                }
            }) {
                error!("failed to start rdev listener due to error: {:#?}", e);
            }
        });
        let window_state = WindowState {
            size,
            position,
            framebuffer_size,
            scale,
            cursor_position,
            glfw_time,
            previous_fps_reset: glfw_time,
            ..Default::default()
        };
        info!("{:#?}", &window_state);
        Ok(Self {
            window,
            events,
            glfw,
            window_state,
            _rdev_thread: (rdev_thread_handle, rdev_thread_signaller),
            rdev_events: receiver,
        })
    }

    #[tracing::instrument]
    pub fn set_framebuffer_size(&mut self, width: u32, height: u32) {
        self.window.set_size(width as i32, height as i32);
    }

    #[tracing::instrument]
    pub fn set_inner_position(&mut self, xpos: i32, ypos: i32) {
        self.window.set_pos(xpos, ypos);
    }

    #[tracing::instrument]
    pub fn set_decorations(&mut self, decorated: bool) {
        self.window.set_decorated(decorated);
    }

    #[tracing::instrument]
    pub fn set_passthrough(&mut self, passthrough: bool) {
        self.window.set_mouse_passthrough(passthrough);
    }
    pub fn set_always_on_top(&mut self, top: bool) {
        self.window.set_floating(top);
    }
    pub fn set_text_clipboard(&mut self, s: &str) {
        tracing::debug!("setting clipboard to: {}", s);
        self.window.set_clipboard_string(s);
    }

    #[tracing::instrument]
    pub fn get_text_clipboard(&mut self) -> Option<String> {
        let t = self.window.get_clipboard_string();
        tracing::debug!("getting clipboard contents. contents: {:?}", &t);
        t
    }

    #[tracing::instrument]
    pub fn should_close(&mut self) -> bool {
        self.window.should_close()
    }

    #[tracing::instrument]
    pub fn attach_to_gw2window(&mut self, new_windim: WindowDimensions) {
        self.set_inner_position(new_windim.x, new_windim.y);
        self.set_framebuffer_size(new_windim.width as u32, new_windim.height as u32);
    }
    fn set_window_hints(glfw: &mut Glfw) {
        // glfw creates opengl context by default. so, we explicitly ask it to not do that.
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        glfw.window_hint(glfw::WindowHint::Floating(true));

        glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));
        glfw.window_hint(glfw::WindowHint::MousePassthrough(true));

        glfw.window_hint(glfw::WindowHint::Decorated(false));
    }

    pub fn tick(&mut self) -> anyhow::Result<RawInput> {
        self.glfw.poll_events();
        let cursor_position = self
            .window
            .get_cursor_pos()
            .pipe(|cp| Vec2::new(cp.0 as f32, cp.1 as f32));
        self.window_state.glfw_time = self.glfw.get_time();
        self.window_state.present_time = OffsetDateTime::now_utc();
        let delta = self.window_state.glfw_time - self.window_state.previous_fps_reset;
        self.window_state.current_frame_number += 1;
        if delta > 1.0 {
            self.window_state.average_frame_rate = self.window_state.current_frame_number;
            self.window_state.previous_fps_reset = self.window_state.glfw_time;
            self.window_state.current_frame_number = 0;
        }

        let mut input = RawInput {
            time: Some(self.window_state.glfw_time),
            ..Default::default()
        };
        if cursor_position != self.window_state.cursor_position {
            self.window_state.cursor_position = cursor_position;
            input.events.push(Event::PointerMoved(
                [cursor_position.x, cursor_position.y].into(),
            ))
        }
        self.window_state.latest_local_events.clear();
        self.window_state.latest_global_events.clear();
        for event in self.rdev_events.try_iter() {
            self.window_state.latest_global_events.push(event);
        }
        for (_, event) in glfw::flush_messages(&self.events) {
            self.window_state.latest_local_events.push(event.clone());
            if let Some(ev) = match event {
                glfw::WindowEvent::FramebufferSize(w, h) => {
                    self.window_state.framebuffer_size = WindowState::i32_to_u32((w, h))?;
                    input.screen_rect = Some(egui::Rect::from_two_pos(
                        Default::default(),
                        [w as f32, h as f32].into(),
                    ));
                    tracing::debug!("window framebuffer size update: {} {}", w, h);
                    None
                }
                glfw::WindowEvent::MouseButton(mb, a, m) => {
                    let emb = Event::PointerButton {
                        pos: cursor_position.conv::<[f32; 2]>().into(),
                        button: glfw_to_egui_pointer_button(mb),
                        pressed: glfw_to_egui_action(a),
                        modifiers: glfw_to_egui_modifers(m),
                    };
                    tracing::trace!("mouse button press: {:?}", &emb);
                    Some(emb)
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    Some(Event::PointerMoved([x as f32, y as f32].into()))
                }
                glfw::WindowEvent::Scroll(x, y) => Some(Event::Scroll([x as f32, y as f32].into())),
                glfw::WindowEvent::Key(k, _, a, m) => match k {
                    glfw::Key::C => {
                        if glfw_to_egui_action(a) && m.contains(glfw::Modifiers::Control) {
                            tracing::trace!("copy event. active modifiers: {:?}", m);
                            Some(Event::Copy)
                        } else {
                            None
                        }
                    }
                    glfw::Key::X => {
                        if glfw_to_egui_action(a) && m.contains(glfw::Modifiers::Control) {
                            tracing::trace!("cut event. active modifiers: {:?}", m);

                            Some(Event::Cut)
                        } else {
                            None
                        }
                    }
                    glfw::Key::V => {
                        if glfw_to_egui_action(a) && m.contains(glfw::Modifiers::Control) {
                            Some(Event::Text(
                                self.window.get_clipboard_string().unwrap_or_default(),
                            ))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
                .or_else(|| {
                    glfw_to_egui_key(k).map(|key| Event::Key {
                        key,
                        pressed: glfw_to_egui_action(a),
                        modifiers: glfw_to_egui_modifers(m),
                    })
                }),
                glfw::WindowEvent::Char(c) => {
                    tracing::trace!("char event: {}", c);
                    Some(Event::Text(c.to_string()))
                }
                glfw::WindowEvent::ContentScale(x, y) => {
                    tracing::warn!("content scale event: {}", x);
                    input.pixels_per_point = Some(x);
                    self.window_state.scale = [x, y].into();
                    None
                }
                glfw::WindowEvent::Close => {
                    tracing::warn!("close event received");
                    anyhow::bail!("close event");
                }
                glfw::WindowEvent::Pos(x, y) => {
                    tracing::debug!("window position changed. {} {}", x, y);
                    self.window_state.position = I32Vec2::new(x, y);
                    None
                }
                glfw::WindowEvent::Size(x, y) => {
                    tracing::debug!("window size changed. {} {}", x, y);
                    self.window_state.size = WindowState::i32_to_u32((x, y))?;
                    None
                }
                glfw::WindowEvent::Refresh => {
                    tracing::debug!("refresh event");
                    None
                }
                glfw::WindowEvent::Focus(f) => {
                    tracing::trace!("focus event: {}", f);
                    None
                }
                glfw::WindowEvent::Iconify(i) => {
                    tracing::trace!("iconify event. {}", i);
                    None
                }
                // glfw::WindowEvent::CursorEnter(_) => todo!(),
                // glfw::WindowEvent::CharModifiers(_, _) => todo!(),
                glfw::WindowEvent::FileDrop(f) => {
                    tracing::info!("file dropped. {:#?}", &f);
                    None
                }
                glfw::WindowEvent::Maximize(m) => {
                    tracing::trace!("maximize event: {}", m);
                    None
                }
                _rest => None,
            } {
                input.events.push(ev);
            }
        }
        Ok(input)
    }
}

/// a function to get the matching egui key event for a given glfw key. egui does not support all the keys provided here.
fn glfw_to_egui_key(key: glfw::Key) -> Option<Key> {
    match key {
        glfw::Key::Space => Some(Key::Space),
        glfw::Key::Num0 => Some(Key::Num0),
        glfw::Key::Num1 => Some(Key::Num1),
        glfw::Key::Num2 => Some(Key::Num2),
        glfw::Key::Num3 => Some(Key::Num3),
        glfw::Key::Num4 => Some(Key::Num4),
        glfw::Key::Num5 => Some(Key::Num5),
        glfw::Key::Num6 => Some(Key::Num6),
        glfw::Key::Num7 => Some(Key::Num7),
        glfw::Key::Num8 => Some(Key::Num8),
        glfw::Key::Num9 => Some(Key::Num9),
        glfw::Key::A => Some(Key::A),
        glfw::Key::B => Some(Key::B),
        glfw::Key::C => Some(Key::C),
        glfw::Key::D => Some(Key::D),
        glfw::Key::E => Some(Key::E),
        glfw::Key::F => Some(Key::F),
        glfw::Key::G => Some(Key::G),
        glfw::Key::H => Some(Key::H),
        glfw::Key::I => Some(Key::I),
        glfw::Key::J => Some(Key::J),
        glfw::Key::K => Some(Key::K),
        glfw::Key::L => Some(Key::L),
        glfw::Key::M => Some(Key::M),
        glfw::Key::N => Some(Key::N),
        glfw::Key::O => Some(Key::O),
        glfw::Key::P => Some(Key::P),
        glfw::Key::Q => Some(Key::Q),
        glfw::Key::R => Some(Key::R),
        glfw::Key::S => Some(Key::S),
        glfw::Key::T => Some(Key::T),
        glfw::Key::U => Some(Key::U),
        glfw::Key::V => Some(Key::V),
        glfw::Key::W => Some(Key::W),
        glfw::Key::X => Some(Key::X),
        glfw::Key::Y => Some(Key::Y),
        glfw::Key::Z => Some(Key::Z),
        glfw::Key::Escape => Some(Key::Escape),
        glfw::Key::Enter => Some(Key::Enter),
        glfw::Key::Tab => Some(Key::Tab),
        glfw::Key::Backspace => Some(Key::Backspace),
        glfw::Key::Insert => Some(Key::Insert),
        glfw::Key::Delete => Some(Key::Delete),
        glfw::Key::Right => Some(Key::ArrowRight),
        glfw::Key::Left => Some(Key::ArrowLeft),
        glfw::Key::Down => Some(Key::ArrowDown),
        glfw::Key::Up => Some(Key::ArrowUp),
        glfw::Key::PageUp => Some(Key::PageUp),
        glfw::Key::PageDown => Some(Key::PageDown),
        glfw::Key::Home => Some(Key::Home),
        glfw::Key::End => Some(Key::End),
        _ => None,
    }
}

pub fn glfw_to_egui_modifers(modifiers: glfw::Modifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.contains(glfw::Modifiers::Alt),
        ctrl: modifiers.contains(glfw::Modifiers::Control),
        shift: modifiers.contains(glfw::Modifiers::Shift),
        mac_cmd: false,
        command: modifiers.contains(glfw::Modifiers::Control),
    }
}

pub fn glfw_to_egui_pointer_button(mb: glfw::MouseButton) -> PointerButton {
    match mb {
        glfw::MouseButton::Button1 => PointerButton::Primary,
        glfw::MouseButton::Button2 => PointerButton::Secondary,
        glfw::MouseButton::Button3 => PointerButton::Middle,
        _ => PointerButton::Secondary,
    }
}

pub fn glfw_to_egui_action(a: glfw::Action) -> bool {
    match a {
        Action::Release => false,
        Action::Press => true,
        Action::Repeat => true,
    }
}
