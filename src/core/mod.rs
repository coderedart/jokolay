use std::path::PathBuf;

use flume::{Receiver, Sender};

use crate::{client::JokoClient, config::JokoConfig, core::input::FrameEvents};

use self::{input::InputManager, painter::Renderer, window::glfw_window::OverlayWindow};

pub mod input;
pub mod painter;
pub mod window;

pub struct JokoCore {
    pub im: InputManager,
    pub rr: Renderer,
    pub ow: OverlayWindow,
    pub events_sender: Sender<FrameEvents>,
    pub commands_receiver: Receiver<CoreFrameCommands>,
    pub events_receiver: Receiver<FrameEvents>,
    pub commands_sender: Sender<CoreFrameCommands>,
}

impl JokoCore {
    pub fn new(joko_config: &mut JokoConfig, _assets_path: PathBuf) -> anyhow::Result<Self> {
        let config = joko_config.overlay_window_config;
        let (ow, events, glfw, gl) = OverlayWindow::create(config)?;
        let im = InputManager::new(events, glfw);
        // start setting up egui initial state
        // let ctx = CtxRef::default();
        // if let Ok(f) = fm.egui_cache_path.open_file().map_err(|e| {
        //     log::error!(
        //         "failed to open egui_cache path at {:?} due to error: {:?}",
        //         &fm.egui_cache_path,
        //         &e
        //     );
        //     e
        // }) {
        //     if let Ok(memory) = serde_json::from_reader(f).map_err(|e| {
        //         log::error!(
        //             "failed to parse memory from file {:?} due ot error {:?}",
        //             &fm.egui_cache_path,
        //             &e
        //         );
        //         e
        //     }) {
        //         *ctx.memory() = memory;
        //     }
        // }

        let rr = Renderer::new(gl.clone());
        let (events_sender, events_receiver) = flume::bounded::<FrameEvents>(1000);
        let (commands_sender, commands_receiver) = flume::bounded::<CoreFrameCommands>(1000);
        Ok(Self {
            im,
            rr,
            ow,
            commands_receiver,
            events_sender,
            events_receiver,
            commands_sender,
        })
    }
    pub fn tick(&mut self) -> bool {
        let events = self.im.tick(self.rr.egui_gl.gl.clone(), &mut self.ow);
        self.events_sender.send(events).unwrap();
        // aggregate all commands so that the channel doesn't get full if client is running more often than our main thread.
        let mut commands = CoreFrameCommands::default();
        loop {
            match self.commands_receiver.try_recv() {
                Ok(fc) => {
                    commands
                        .window_commads
                        .extend(fc.window_commads.into_iter());
                    commands
                        .input_commands
                        .extend(fc.input_commands.into_iter());
                    commands
                        .render_commands
                        .extend(fc.render_commands.into_iter());
                }
                Err(e) => match e {
                    flume::TryRecvError::Empty => break,
                    flume::TryRecvError::Disconnected => return true,
                },
            }
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
            }
        }
        for i in commands.input_commands {
            match i {
                InputCommand::SetClipBoard(s) => self.ow.window.set_clipboard_string(&s),
                InputCommand::GetClipBoard(s) => {
                    let _ = s.send(self.ow.window.get_clipboard_string().unwrap_or_default());
                }
            }
        }

        // draw stuff
        self.rr.clear();
        self.ow.swap_buffers();

        true
    }
    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut client =
            JokoClient::new(self.events_receiver.clone(), self.commands_sender.clone())?;
        self.im
            .glfw_input
            .glfw
            .set_swap_interval(glfw::SwapInterval::None);
        let client_thread = std::thread::spawn(move || -> anyhow::Result<()> {
            loop {
                client.tick()?;
            }
        });
        while self.tick() {}
        client_thread.join().unwrap().unwrap();
        Ok(())
    }
}
#[derive(Debug)]
pub enum InputCommand {
    SetClipBoard(String),
    GetClipBoard(tokio::sync::oneshot::Sender<String>),
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
}
#[derive(Debug, Clone)]
pub enum TextureCommand {
    Upload {
        pixels: Vec<u8>,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
    },
    BumpTextureArraySize,
    Reset,
}
#[derive(Debug, Clone)]
pub enum RenderCommand {
    Draw,
}
pub enum GlobalCommand {}

#[derive(Debug, Default)]
pub struct CoreFrameCommands {
    pub window_commads: Vec<WindowCommand>,
    pub input_commands: Vec<InputCommand>,
    pub render_commands: Vec<RenderCommand>,
}
