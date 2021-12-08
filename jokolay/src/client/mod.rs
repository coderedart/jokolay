use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

use crate::{
    client::{am::AssetManager, gui::config_window::ConfigWindow, tc::TextureClient},
    config::JokoConfig,
    core::{
        input::{
            glfw_input::{self, glfw_to_egui_action, glfw_to_egui_key, glfw_to_egui_modifers},
            FrameEvents,
        },
        painter::{egui_renderer::EguiMesh, RenderCommand},
        window::WindowCommand,
        CoreFrameCommands,
    },
};
use egui::{Event, Pos2, RawInput};
use flume::{Receiver, Sender};
use parking_lot::RwLock;

pub mod am;
pub mod gui;
pub mod mlink;
pub mod mm;
pub mod tc;

/// The main client app struct that runs in the off-render thread.
pub struct JokoClient {
    pub tc: TextureClient,
    pub ctx: egui::CtxRef,
    pub am: AssetManager,
    pub config_window: ConfigWindow,
    pub handle: tokio::runtime::Handle,
    pub quit_signal_sender: flume::Sender<()>,
    pub events_receiver: Receiver<FrameEvents>,
    pub commands_sender: Sender<CoreFrameCommands>,
    pub soft_restart: Arc<AtomicBool>,
}

impl JokoClient {
    pub fn new(
        events_receiver: Receiver<FrameEvents>,
        commands_sender: Sender<CoreFrameCommands>,
        soft_restart: Arc<AtomicBool>,
        assets_path: PathBuf,
        config: Arc<RwLock<JokoConfig>>,
    ) -> anyhow::Result<Self> {
        let rt = tokio::runtime::Runtime::new()?;
        let handle = rt.handle().clone();
        let (quit_signal_sender, quit_signal_receiver) = flume::bounded(1);
        std::thread::spawn(move || {
            rt.block_on(async {
                let quit_signal_receiver = quit_signal_receiver.into_recv_async();
                quit_signal_receiver.await.unwrap();
            })
        });
        let am = AssetManager::new(assets_path);
        let mut ctx = egui::CtxRef::default();
        {
            // initializing the first screen_rect because until there's a resize event, egui won't get any input about screen_size and will use something ridiculous like 10,000 x 10,000 as screen rect
            let input = RawInput {
                screen_rect: Some(egui::Rect::from_two_pos(
                    [0.0, 0.0].into(),
                    [
                        config.read().overlay_window_config.framebuffer_width as f32,
                        config.read().overlay_window_config.framebuffer_height as f32,
                    ]
                    .into(),
                )),
                ..Default::default()
            };

            ctx.begin_frame(input);
            let _ = ctx.end_frame();
        }
        Ok(Self {
            tc: TextureClient::new(handle.clone()),
            ctx,
            config_window: ConfigWindow::new(config),
            am,
            handle,
            quit_signal_sender,
            commands_sender,
            events_receiver,
            soft_restart,
        })
    }
    pub fn tick(&mut self) -> anyhow::Result<bool> {
        // collect all the events from main thread
        let mut c = CoreFrameCommands::default();
        // block until we get events so that we don't spin for no reason
        match self.events_receiver.recv() {
            // if we get events, just drain the receiver so that if client is slow, frame_events don't pile up from the core
            // and we also don't want to just keep receiving events form 100 frames back.
            Ok(mut e) => {
                for fe in self.events_receiver.try_iter() {
                    e.average_frame_rate = fe.average_frame_rate;
                    e.cursor_position = fe.cursor_position;
                    e.time = fe.time;
                    e.all_events.extend(fe.all_events.into_iter());
                }
                let mut quit = false;
                let prev_frame_cursor_pos = self.config_window.cursor_position;
                self.config_window.cursor_position = e.cursor_position;
                let new_frame_rate = e.average_frame_rate;
                let i = handle_events(
                    e,
                    &mut quit,
                    prev_frame_cursor_pos,
                    self.config_window.config.clone(),
                );
                if quit {
                    self.quit_signal_sender.send(()).unwrap();
                    return Ok(false);
                }

                // now we start the egui frame

                self.ctx.begin_frame(i);

                self.config_window
                    .tick(self.ctx.clone(), &mut c, new_frame_rate);
            }
            Err(e) => match e {
                flume::RecvError::Disconnected => {
                    self.quit_signal_sender.send(()).unwrap();
                    return Ok(false);
                }
            },
        }
        // check if egui texture needs updating
        self.tc.tick(self.ctx.texture());
        // start filling up the egui with ui

        // start preparing meshes to send to main thread for drawing

        let (output, shapes) = self.ctx.end_frame();
        if let Some(cmds) = self.tc.tex_commands.as_mut() {
            c.render_commands.append(cmds)
        }
        if !output.copied_text.is_empty() {
            // match copypasta::ClipboardContext::new().and_then(|mut cc| cc.set_contents(output.copied_text.clone())) {
            //     Ok(_) => log::debug!("text copied to clipboard. text: {}", output.copied_text),
            //     Err(e) => log::error!("clipboard error: {:?}", &e),
            // }
            c.window_commads
                .push(WindowCommand::SetClipBoard(output.copied_text));
        }
        {
            let screen_size = self.ctx.input().screen_rect();
            let allo = self.tc.get_alloc_tex(egui::TextureId::Egui).unwrap();
            let tex_coords = allo.get_tex_coords();
            let shapes = self.ctx.tessellate(shapes);
            let meshes: Vec<EguiMesh> = shapes
                .into_iter()
                .map(|cm| EguiMesh {
                    sampler: 0,
                    sampler_layer: tex_coords.layer as i32,
                    tcx_offset: tex_coords.startx,
                    tcy_offset: tex_coords.starty,
                    tcx_scale: tex_coords.scalex,
                    tcy_scale: tex_coords.scaley,
                    screen_size: [screen_size.width(), screen_size.height()].into(),
                    clip_rect: cm.0,
                    vertices: cm.1.vertices,
                    indices: cm.1.indices,
                    tid: cm.1.texture_id,
                })
                .collect();
            c.render_commands
                .push(RenderCommand::UpdateEguiScene(meshes));
        }
        if self.ctx.wants_pointer_input() || self.ctx.wants_keyboard_input() {
            c.window_commads.push(WindowCommand::Passthrough(false));
        } else {
            c.window_commads.push(WindowCommand::Passthrough(true));
        }
        self.commands_sender.send(c).unwrap();
        // std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(true)
    }
    pub fn run(mut self) {
        while self.tick().unwrap() {}
    }
}

pub fn handle_events(
    events: FrameEvents,
    quit: &mut bool,
    previous_cursor_position: Pos2,
    config: Arc<RwLock<JokoConfig>>,
) -> RawInput {
    let mut input = RawInput {
        time: Some(events.time),
        ..Default::default()
    };
    let mut jc = config.write();
    if events.cursor_position != previous_cursor_position {
        input
            .events
            .push(egui::Event::PointerMoved(events.cursor_position));
    }
    if let Some(clipboard_text) = events.clipboard_text {
        log::trace!("paste event: {}", &clipboard_text);
        input.events.push(Event::Text(clipboard_text));
    }
    for e in events.all_events {
        if let Some(ev) = match e {
            glfw::WindowEvent::FramebufferSize(w, h) => {
                input.screen_rect = Some(egui::Rect::from_two_pos(
                    Default::default(),
                    Pos2 {
                        x: w as f32,
                        y: h as f32,
                    },
                ));
                log::trace!("window framebuffer size update: {} {}", w, h);
                jc.overlay_window_config.framebuffer_width = w.try_into().unwrap();
                jc.overlay_window_config.framebuffer_height = h.try_into().unwrap();
                None
            }
            glfw::WindowEvent::MouseButton(mb, a, m) => {
                let emb = Event::PointerButton {
                    pos: events.cursor_position,
                    button: glfw_input::glfw_to_egui_pointer_button(mb),
                    pressed: glfw_to_egui_action(a),
                    modifiers: glfw_to_egui_modifers(m),
                };
                log::trace!("mouse button press: {:?}", &emb);
                Some(emb)
            }
            glfw::WindowEvent::CursorPos(x, y) => {
                Some(Event::PointerMoved([x as f32, y as f32].into()))
            }
            glfw::WindowEvent::Scroll(x, y) => {
                input.scroll_delta = [
                    x as f32 * jc.input_config.scroll_power,
                    y as f32 * jc.input_config.scroll_power,
                ]
                .into();
                None
            }
            glfw::WindowEvent::Key(k, scan_code, a, m) => {
                log::trace!(
                    "key: {:?}, scan_code: {:?}, action: {:?}, modifiers: {:?}",
                    k,
                    scan_code,
                    a,
                    m
                );
                match k {
                    glfw::Key::C => {
                        if glfw_to_egui_action(a) && m.contains(glfw::Modifiers::Control) {
                            log::trace!("copy event. active modifiers: {:?}", m);
                            Some(Event::Copy)
                        } else {
                            None
                        }
                    }
                    glfw::Key::X => {
                        if glfw_to_egui_action(a) && m.contains(glfw::Modifiers::Control) {
                            log::trace!("cut event. active modifiers: {:?}", m);

                            Some(Event::Cut)
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
                })
            }
            glfw::WindowEvent::Char(c) => {
                log::trace!("char event: {}", c);
                Some(Event::Text(c.to_string()))
            }
            glfw::WindowEvent::ContentScale(x, _) => {
                log::warn!("content scale event: {}", x);
                input.pixels_per_point = Some(x);
                None
            }
            glfw::WindowEvent::Close => {
                log::warn!("close event received");
                *quit = true;
                None
            }
            glfw::WindowEvent::Pos(x, y) => {
                log::debug!("window position changed. {} {}", x, y);
                jc.overlay_window_config.window_pos_x = x;
                jc.overlay_window_config.window_pos_y = y;
                None
            }
            glfw::WindowEvent::Size(x, y) => {
                log::debug!("window size changed. {} {}", x, y);
                None
            }
            glfw::WindowEvent::Refresh => {
                log::debug!("refresh event");
                None
            }
            glfw::WindowEvent::Focus(f) => {
                log::trace!("focus event: {}", f);
                None
            }
            glfw::WindowEvent::Iconify(i) => {
                log::trace!("iconify event. {}", i);
                None
            }
            // glfw::WindowEvent::CursorEnter(_) => todo!(),
            // glfw::WindowEvent::CharModifiers(_, _) => todo!(),
            glfw::WindowEvent::FileDrop(f) => {
                log::info!("file dropped. {:#?}", &f);
                None
            }
            glfw::WindowEvent::Maximize(m) => {
                log::trace!("maximize event: {}", m);
                None
            }
            _rest => None,
        } {
            input.events.push(ev);
        }
    }
    input
}
