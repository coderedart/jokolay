use egui::{Event, Pos2, RawInput};
use flume::{Receiver, Sender};

use crate::{
    client::{gui::main_window::MainWindow, tc::atlas::TextureClient},
    core::{
        input::{
            glfw_input::{self, glfw_to_egui_action, glfw_to_egui_key, glfw_to_egui_modifers},
            FrameEvents,
        },
        painter::{egui_renderer::EguiMesh, RenderCommand},
        CoreFrameCommands,
    },
};

pub mod gui;
pub mod mlink;
pub mod tc;

pub struct JokoClient {
    pub tc: TextureClient,
    pub ctx: egui::CtxRef,
    pub main_window: MainWindow,
    pub handle: tokio::runtime::Handle,
    pub quit_signal_sender: tokio::sync::oneshot::Sender<()>,
    pub events_receiver: Receiver<FrameEvents>,
    pub commands_sender: Sender<CoreFrameCommands>,
}

impl JokoClient {
    pub fn new(
        events_receiver: Receiver<FrameEvents>,
        commands_sender: Sender<CoreFrameCommands>,
    ) -> anyhow::Result<Self> {
        let rt = tokio::runtime::Runtime::new()?;
        let handle = rt.handle().clone();
        let (quit_signal_sender, quit_signal_receiver) = tokio::sync::oneshot::channel::<()>();
        std::thread::spawn(move || {
            rt.block_on(async {
                quit_signal_receiver.await.unwrap();
            })
        });
        Ok(Self {
            tc: TextureClient::new(handle.clone()),
            ctx: egui::CtxRef::default(),
            main_window: MainWindow::default(),
            handle,
            quit_signal_sender,
            commands_sender,
            events_receiver,
        })
    }
    pub fn tick(&mut self) -> anyhow::Result<()> {
        match self.events_receiver.recv() {
            Ok(e) => {
                let i = handle_events(e);
                self.ctx.begin_frame(i);
            }
            Err(e) => match e {
                flume::RecvError::Disconnected => todo!(),
            },
        }
        egui::Window::new("w")
        .scroll2([true, true])
        .show(&self.ctx.clone(), |ui| {
            self.ctx.settings_ui(ui);
        });

        self.tc.tick(self.ctx.texture());

        let screen_size = self.ctx.input().screen_rect();
        let allo = self.tc.get_alloc_tex(egui::TextureId::Egui).unwrap();
        let tex_coords = allo.get_tex_coords();
        let (output, shapes) = self.ctx.end_frame();
        let mut c = CoreFrameCommands::default();

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
        if let Some(tcmd) = self.tc.tex_commands.take() {
            for cmd in tcmd.iter() {
                match cmd {
                    crate::core::TextureCommand::Upload {
                        pixels: _,
                        x_offset: _,
                        y_offset: _,
                        z_offset: _,
                        width,
                        height,
                    } => {
                        dbg!(width, height);
                    }
                    crate::core::TextureCommand::BumpTextureArraySize => {
                        dbg!("bump");
                    }
                    crate::core::TextureCommand::Reset => todo!(),
                }
            }
            c.texture_commands = tcmd;
        }
        self.commands_sender.send(c).unwrap();
        Ok(())
    }
}

pub fn handle_events(events: FrameEvents) -> RawInput {
    let mut input = RawInput::default();
    input.time = Some(events.time);
    input
        .events
        .push(egui::Event::PointerMoved(events.cursor_position));
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
                std::process::exit(0);
            }
            _ => None,
        } {
            input.events.push(ev);
        }
    }
    input
}
