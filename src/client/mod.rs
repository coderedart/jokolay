use std::{path::PathBuf, sync::{atomic::AtomicBool, Arc}};

use egui::{Event, Pos2, RawInput};
use flume::{Receiver, Sender};

use crate::{client::{am::AssetManager, gui::{debug::DebugWindow, main_window::MainWindow}, mm::MarkerManager, tc::TextureClient}, core::{CoreFrameCommands, input::{
            glfw_input::{self, glfw_to_egui_action, glfw_to_egui_key, glfw_to_egui_modifers},
            FrameEvents,
        }, painter::{egui_renderer::EguiMesh, RenderCommand}, window::WindowCommand}};

pub mod gui;
pub mod mlink;
pub mod tc;
pub mod am;
pub mod mm;

pub struct JokoClient {
    pub tc: TextureClient,
    pub ctx: egui::CtxRef,
    pub am: AssetManager,
    pub main_window: MainWindow,
    pub mm: MarkerManager,
    pub handle: tokio::runtime::Handle,
    pub quit_signal_sender: flume::Sender<()>,
    pub events_receiver: Receiver<FrameEvents>,
    pub commands_sender: Sender<CoreFrameCommands>,
    pub soft_restart: Arc<AtomicBool>,
    #[cfg(debug_assertions)]
    pub debug_window: DebugWindow,
    pub mouse_position: Pos2,
}

impl JokoClient {
    pub fn new(
        events_receiver: Receiver<FrameEvents>,
        commands_sender: Sender<CoreFrameCommands>,
        soft_restart: Arc<AtomicBool>,
        assets_path: PathBuf
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
        let             am = AssetManager::new(assets_path);

        let mut mm = MarkerManager::default();

        Ok(Self {
            tc: TextureClient::new(handle.clone()),
            ctx: egui::CtxRef::default(),
            main_window: MainWindow::default(),
            am,
            mm,
            handle,
            quit_signal_sender,
            commands_sender,
            events_receiver,
            soft_restart,
            #[cfg(debug_assertions)]
            debug_window: Default::default(),
            mouse_position: Pos2::default()
        })
    }
    pub fn tick(&mut self) -> anyhow::Result<bool> {
        let mut c = CoreFrameCommands::default();
        // block until we get events so that we don't spin for no reason
        match self.events_receiver.recv() {
            // if we get events, just drain the receiver so that if client is slow, frame_events don't pile up from the core
            // and we also don't want to just keep receiving events form 100 frames back.
            Ok(mut e) => {
                for fe in self.events_receiver.try_iter() {
                    e.average_frame_rate = e.average_frame_rate;
                    e.cursor_position = fe.cursor_position;
                    e.time = fe.time;
                    e.all_events.extend(fe.all_events.into_iter());
                }
                let average_fps = e.average_frame_rate;
                let mut quit = false;
                if e.all_events.is_empty() && self.mouse_position == e.cursor_position {
                    // return Ok(true);
                }
                let prev_frame_cursor_pos = self.mouse_position;
                self.mouse_position = e.cursor_position;

                let i = handle_events(e, &mut quit, prev_frame_cursor_pos);
                if quit {
                    self.quit_signal_sender.send(()).unwrap();
                    return Ok(false);
                }
               
                // now we start the egui frame
                self.ctx.begin_frame(i);
                
                #[cfg(debug_assertions)]
                gui::debug::show_debug_window(self.ctx.clone(), self, average_fps, &mut c);
            }
            Err(e) => match e {
                flume::RecvError::Disconnected => {
                    self.quit_signal_sender.send(()).unwrap();
                    return Ok(false);
                }
            },
        }

        

        self.tc.tick(self.ctx.texture());
        
        let screen_size = self.ctx.input().screen_rect();
        let allo = self.tc.get_alloc_tex(egui::TextureId::Egui).unwrap();
        let tex_coords = allo.get_tex_coords();
        let (output, shapes) = self.ctx.end_frame();
        self.tc
            .tex_commands
            .as_mut()
            .map(|cmd| c.render_commands.append(cmd));
        if output.needs_repaint {
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

pub fn handle_events(events: FrameEvents, quit: &mut bool, previous_cursor_position: Pos2) -> RawInput {
    let mut input = RawInput::default();
    input.time = Some(events.time);
    if events.cursor_position != previous_cursor_position {
        input
        .events
        .push(egui::Event::PointerMoved(events.cursor_position));
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
                None
            }
            glfw::WindowEvent::MouseButton(mb, a, m) => Some(Event::PointerButton {
                pos: events.cursor_position,
                button: glfw_input::glfw_to_egui_pointer_button(mb),
                pressed: glfw_to_egui_action(a),
                modifiers: glfw_to_egui_modifers(m),
            }),
            glfw::WindowEvent::CursorPos(x, y) => {
                Some(Event::PointerMoved([x as f32, y as f32].into()))
            }
            glfw::WindowEvent::Scroll(x, y) => {
                input.scroll_delta = [x as f32, y as f32].into();
                None
            }
            glfw::WindowEvent::Key(k, _, a, m) => {
                if let Some(key) = glfw_to_egui_key(k) {
                    Some(Event::Key {
                        key: key,
                        pressed: glfw_to_egui_action(a),
                        modifiers: glfw_to_egui_modifers(m),
                    })
                } else {
                    None
                }
            }
            glfw::WindowEvent::ContentScale(x, _) => {
                input.pixels_per_point = Some(x);
                None
            }
            glfw::WindowEvent::Close => {
                *quit = true;
                None
            }
            _ => None,
        } {
            input.events.push(ev);
        }
    }
    input
}
