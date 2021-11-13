use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use flume::{Receiver, Sender};

use crate::{
    config::JokoConfig,
    core::{input::FrameEvents, painter::RenderCommand, window::WindowCommand},
};

use self::{input::InputManager, painter::Renderer, window::OverlayWindow};

pub mod input;
pub mod painter;
pub mod window;

#[derive(Debug)]
pub struct JokoCore {
    pub im: InputManager,
    pub rr: Renderer,
    pub ow: OverlayWindow,
    pub events_sender: Sender<FrameEvents>,
    pub commands_receiver: Receiver<CoreFrameCommands>,
}

impl JokoCore {
    /// creates a Core with windowing, input and rendering objects along with the communication channels for interacting with the client.
    pub fn new(
        joko_config: &mut JokoConfig,
        _assets_path: PathBuf,
        commands_receiver: Receiver<CoreFrameCommands>,
        events_sender: Sender<FrameEvents>,
    ) -> anyhow::Result<Self> {
        let config = joko_config.overlay_window_config;
        let (ow, events, glfw, gl) = OverlayWindow::create(config)?;
        let im = InputManager::new(events, glfw);
        let rr = Renderer::new(gl.clone());

        Ok(Self {
            im,
            rr,
            ow,
            commands_receiver,
            events_sender,
        })
    }
    pub fn tick(&mut self) -> bool {
        let events = self.im.tick(self.rr.gl.clone(), &mut self.ow);
        match self.events_sender.send(events) {
            Ok(_) => {}
            Err(_) => return false,
        }
        // aggregate all commands so that the channel doesn't get full if client is running more often than our main thread.
        let mut commands = CoreFrameCommands::default();
        for fc in self.commands_receiver.try_iter() {
            commands
                .window_commads
                .extend(fc.window_commads.into_iter());
            commands
                .render_commands
                .extend(fc.render_commands.into_iter());
        }
        for wc in commands.window_commads {
            match wc {
                WindowCommand::Resize(w, h) => self.ow.set_framebuffer_size(w, h),
                WindowCommand::Repos(x, y) => self.ow.set_inner_position(x, y),
                WindowCommand::Transparent(_) => {}
                WindowCommand::Passthrough(p) => self.ow.set_passthrough(p),
                WindowCommand::Decorated(d) => self.ow.set_decorations(d),
                WindowCommand::AlwaysOnTop(f) => self.ow.window.set_floating(f),
                WindowCommand::ShouldClose(b) => self.ow.window.set_should_close(b),
                WindowCommand::SwapInterval(i) => self.im.glfw_input.glfw.set_swap_interval(i),
                WindowCommand::SetTransientFor(_) => todo!(),
                WindowCommand::SetClipBoard(s) => self.ow.window.set_clipboard_string(&s),
                WindowCommand::GetClipBoard(s) => {
                    let _ = s.send(self.ow.window.get_clipboard_string().unwrap_or_default());
                }
            }
        }

        for r in commands.render_commands {
            match r {
                RenderCommand::UpdateEguiScene(meshes) => self.rr.update_egui_scene(meshes),
                RenderCommand::TextureUpload {
                    pixels,
                    x_offset,
                    y_offset,
                    z_offset,
                    width,
                    height,
                } => self.rr.ts.upload_pixels(
                    &pixels,
                    x_offset,
                    y_offset,
                    z_offset,
                    width as u32,
                    height as u32,
                ),
                RenderCommand::BumpTextureArraySize => self.rr.ts.bump_tex_array_size(None),
                RenderCommand::Reset => todo!(),
            }
        }
        // draw stuff
        self.rr.clear();
        self.rr.draw_egui();
        self.ow.swap_buffers();

        true
    }
    pub fn run(mut self) -> bool {
        let mut t = Instant::now();
        let mut frame_number = 0;
        while self.tick() {
            if t.elapsed() > Duration::from_secs(1) {
                t = Instant::now();
                dbg!(frame_number);
                frame_number = 0;
            }
            frame_number += 1;
        }
        false
    }
}

#[derive(Debug, Default)]
pub struct CoreFrameCommands {
    pub window_commads: Vec<WindowCommand>,
    pub render_commands: Vec<RenderCommand>,
}
