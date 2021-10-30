use flume::{Receiver, Sender};

use crate::{client::tc::atlas::TextureClient, core::{CoreFrameCommands, RenderCommand, WindowCommand, input::FrameEvents}};


pub mod tc;
pub mod gui;
pub mod mlink;

pub struct JokoClient {
    pub tc: TextureClient,
    pub handle: tokio::runtime::Handle,
    pub quit_signal_sender: tokio::sync::oneshot::Sender<()>,
    pub events_receiver: Receiver<FrameEvents>,
    pub commands_sender: Sender<CoreFrameCommands>
}

impl JokoClient {
    pub fn new( events_receiver: Receiver<FrameEvents>, commands_sender: Sender<CoreFrameCommands>) -> anyhow::Result<Self> {
        let rt = tokio::runtime::Runtime::new()?;
        let handle = rt.handle().clone();
        let( quit_signal_sender, quit_signal_receiver) = tokio::sync::oneshot::channel::<()>();
        std::thread::spawn(move || {
            rt.block_on(async {
                quit_signal_receiver.await.unwrap();
            })
        });
        Ok(Self {
            tc: TextureClient::new(handle.clone()),
            handle,
            quit_signal_sender,
            commands_sender,
            events_receiver
        })
    }
    pub fn tick(&mut self) -> anyhow::Result<()>
    {
        match self.events_receiver.recv() {
            Ok(e) => {
                if !e.all_events.is_empty() {
                    dbg!(e);
                };
            },
            Err(e) => match e {
                flume::RecvError::Disconnected => todo!(),
            },
        }
        let c = CoreFrameCommands::default();
        match self.commands_sender.send(c) {
            Ok(_) => {},
            Err(_s) => {},
        }
        Ok(())
    }
}