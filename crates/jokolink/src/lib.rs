//! Jokolink is a crate to deal with Mumble Link data exposed by games/apps on windows via shared memory

//! Joko link is designed to primarily get the MumbleLink or the window size
//! of the GW2 window for Jokolay (an crossplatform overlay for Guild Wars 2).
//! on windows, you can use it to create/open shared memory.
//! and on linux, you can run jokolink binary in wine, which will create/open shared memory and copy-paste it into /dev/shm.
//! then, you can easily read the /dev/shm file from a any number of linux native applications.
//! along with mumblelink data, it also copies the x11 window id of gw2. you can use this to get the size of gw2 window.
//!

mod mumble;
use egui::DragValue;
use enumflags2::BitFlags;
use glam::IVec2;
use miette::{IntoDiagnostic, Result, WrapErr};
pub use mumble::*;
use serde_json::from_str;
use std::sync::Arc;
use tracing::error;

/// The default mumble link name. can only be changed by passing the `-mumble` options to gw2 for multiboxing
pub const DEFAULT_MUMBLELINK_NAME: &str = "MumbleLink";
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod win;

#[cfg(target_os = "linux")]
use linux::MumbleLinuxImpl as MumblePlatformImpl;
#[cfg(target_os = "windows")]
use win::MumbleWinImpl as MumblePlatformImpl;
// Useful link size is only [ctypes::USEFUL_C_MUMBLE_LINK_SIZE] . And we add 100 more bytes so that jokolink can put some extra stuff in there
// pub(crate) const JOKOLINK_MUMBLE_BUFFER_SIZE: usize = ctypes::USEFUL_C_MUMBLE_LINK_SIZE + 100;
/// This primarily manages the mumble backend.
/// the purpose of `MumbleBackend` is to get mumble link data and window dimensions when asked.
/// Manager also caches the previous mumble link details like window dimensions or mapid etc..
/// and every frame gets the latest mumble link data, and compares with the previous frame.
/// if any of the changed this frame, it will set the relevant changed flags so that plugins
/// or other parts of program which care can run the relevant code.
pub struct MumbleManager {
    /// This abstracts over the windows and linux impl of mumble link functionality.
    /// we use this to get the latest mumble link and latest window dimensions of the current mumble link
    backend: MumblePlatformImpl,
    /// latest mumble link
    link: Arc<MumbleLink>,
    show_window: bool,
}
impl MumbleManager {
    pub fn new(name: &str, _jokolay_window_id: Option<u32>) -> Result<Self> {
        let backend = MumblePlatformImpl::new(name)?;
        Ok(Self {
            backend,
            link: Arc::new(Default::default()),
            show_window: true,
        })
    }
    pub fn tick(&mut self, ctx: &egui::Context) -> Result<Option<Arc<MumbleLink>>> {
        if let Err(e) = self.backend.tick() {
            error!(?e, "mumble backend tick error");
            return Ok(None);
        }

        if !self.backend.is_alive() {
            // reset link
            if self.link.ui_tick != 0 {
                self.link = Arc::new(Default::default());
            }
            return Ok(None);
        }
        // backend is alive and tick is successful. time to get link
        let cml: ctypes::CMumbleLink = self.backend.get_cmumble_link();
        if cml.ui_tick == 0 && self.link.ui_tick != 0 {
            self.link = Arc::new(Default::default());
        }

        if cml.ui_tick == 0 || cml.context.client_pos_size == [0; 4] {
            return Ok(None);
        }
        let mut changes: BitFlags<MumbleChanges> = Default::default();
        // safety. as the link is valid, we can use as_ref
        let json_string = widestring::U16CStr::from_slice_truncate(&cml.identity)
            .into_diagnostic()
            .wrap_err("failed to get widestring out of cml identity")?
            .to_string()
            .into_diagnostic()
            .wrap_err("failed to convert widestring to cstring")?;

        let identity: ctypes::CIdentity = from_str(&json_string)
            .into_diagnostic()
            .wrap_err("failed to deserialize identity from json string")?;
        let uisz = identity
            .get_uisz()
            .ok_or(miette::miette!("uisz is invalid"))?;

        if self.link.ui_tick != cml.ui_tick {
            changes.insert(MumbleChanges::UiTick);
        }
        if self.link.name != identity.name {
            changes.insert(MumbleChanges::Character);
        }
        if self.link.map_id != cml.context.map_id {
            changes.insert(MumbleChanges::Map);
        }
        // let window_pos = IVec2::new(
        //     cml.context.window_pos_size[0],
        //     cml.context.window_pos_size[1],
        // );
        // let window_size = IVec2::new(
        //     cml.context.window_pos_size[2],
        //     cml.context.window_pos_size[3],
        // );
        // let window_pos_without_borders = IVec2::new(
        //     cml.context.window_pos_size_without_borders[0],
        //     cml.context.window_pos_size_without_borders[1],
        // );
        // let window_size_without_borders = IVec2::new(
        //     cml.context.window_pos_size_without_borders[2],
        //     cml.context.window_pos_size_without_borders[3],
        // );
        let client_pos = IVec2::new(
            cml.context.client_pos_size[0],
            cml.context.client_pos_size[1],
        );
        let client_size = IVec2::new(
            cml.context.client_pos_size[2],
            cml.context.client_pos_size[3],
        );

        if self.link.client_pos != client_pos {
            changes.insert(MumbleChanges::WindowPosition);
        }
        if self.link.client_size != client_size {
            changes.insert(MumbleChanges::WindowSize);
        }
        let link = Arc::new(MumbleLink {
            ui_tick: cml.ui_tick,
            f_avatar_position: cml.f_avatar_position.into(),
            f_avatar_front: cml.f_avatar_front.into(),
            f_camera_position: cml.f_camera_position.into(),
            f_camera_front: cml.f_camera_front.into(),
            name: identity.name,
            map_id: cml.context.map_id,
            fov: identity.fov,
            uisz,
            // window_pos,
            // window_size,
            changes,
            // window_pos_without_borders,
            // window_size_without_borders,
            dpi_scaling: cml.context.dpi_scaling,
            dpi: cml.context.dpi,
            client_pos,
            client_size,
        });
        self.link = link.clone();
        egui::Window::new("Mumble Manager")
            .open(&mut self.show_window)
            .show(ctx, |ui| {
                if link.ui_tick == 0 {
                    ui.label("Mumble is not initialized");
                } else {
                    let link: MumbleLink = link.as_ref().clone();
                    mumble_ui(ui, link);
                }
            });
        Ok(if self.link.ui_tick == 0 {
            None
        } else {
            Some(self.link.clone())
        })
    }
}

fn mumble_ui(ui: &mut egui::Ui, mut link: MumbleLink) {
    egui::Grid::new("link grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("ui tick");
            ui.add(DragValue::new(&mut link.ui_tick));
            ui.end_row();
            ui.label("fov");
            ui.add(DragValue::new(&mut link.fov));
            ui.end_row();
            ui.label("w/h ratio");
            let ratio = link.client_size.as_vec2();
            let mut ratio = ratio.x / ratio.y;
            ui.add(DragValue::new(&mut ratio));
            ui.end_row();
            ui.label("character");
            ui.label(&link.name);
            ui.end_row();
            ui.label("map id");
            ui.add(DragValue::new(&mut link.map_id));
            ui.end_row();
            ui.label("client pos");
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut link.client_pos.x));
                ui.add(DragValue::new(&mut link.client_pos.y));
            });
            ui.end_row();
            ui.label("client size");
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut link.client_size.x));
                ui.add(DragValue::new(&mut link.client_size.y));
            });
            ui.end_row();
            ui.label("dpi scaling");
            ui.add(DragValue::new(&mut link.dpi_scaling));
            ui.end_row();
            ui.label("dpi");
            ui.add(DragValue::new(&mut link.dpi));
            ui.end_row();

            // ui.label("position");
            // ui.horizontal(|ui| {
            //     ui.add(DragValue::new(&mut link.window_pos.x));
            //     ui.add(DragValue::new(&mut link.window_pos.y));
            // });
            // ui.end_row();
            // ui.label("size");
            // ui.horizontal(|ui| {
            //     ui.add(DragValue::new(&mut link.window_size.x));
            //     ui.add(DragValue::new(&mut link.window_size.y));
            // });
            // ui.end_row();
            // ui.label("position_nb");
            // ui.horizontal(|ui| {
            //     ui.add(DragValue::new(&mut link.window_pos_without_borders.x));
            //     ui.add(DragValue::new(&mut link.window_pos_without_borders.y));
            // });
            // ui.end_row();
            // ui.label("size_nb");
            // ui.horizontal(|ui| {
            //     ui.add(DragValue::new(&mut link.window_size_without_borders.x));
            //     ui.add(DragValue::new(&mut link.window_size_without_borders.y));
            // });
            // ui.end_row();
        });
}
