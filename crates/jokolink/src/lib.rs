//! Jokolink is a crate to deal with Mumble Link data exposed by games/apps on windows via shared memory

//! Joko link is designed to primarily get the MumbleLink or the window size
//! of the GW2 window for Jokolay (an crossplatform overlay for Guild Wars 2).
//! on windows, you can use it to create/open shared memory.
//! and on linux, you can run jokolink binary in wine, which will create/open shared memory and copy-paste it into /dev/shm.
//! then, you can easily read the /dev/shm file from a any number of linux native applications.
//! along with mumblelink data, it also copies the x11 window id of gw2. you can use this to get the size of gw2 window.
//!

mod mumble;
use joko_core::prelude::*;
pub use mumble::*;

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
            error!("mumble backend tick error: {e:#?}");
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

        if cml.ui_tick == 0 || cml.context.window_pos_size == [0; 4] {
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
        let window_pos = IVec2::new(
            cml.context.window_pos_size[0],
            cml.context.window_pos_size[1],
        );
        let window_size = IVec2::new(
            cml.context.window_pos_size[2],
            cml.context.window_pos_size[3],
        );
        if self.link.window_pos != window_pos {
            changes.insert(MumbleChanges::WindowPosition);
        }
        if self.link.window_size != window_size {
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
            window_pos,
            window_size,
            changes,
        });
        self.link = link.clone();
        egui::Window::new("Mumble Manager")
            .open(&mut self.show_window)
            .show(ctx, |ui| {
                if link.ui_tick == 0 {
                    ui.label("Mumble is not initialized");
                } else {
                    mumble_ui(ui, &link);
                }
            });
        Ok(if self.link.ui_tick == 0 {
            None
        } else {
            Some(self.link.clone())
        })
    }
}

fn mumble_ui(ui: &mut egui::Ui, link: &MumbleLink) {
    egui::Grid::new("link grid").num_columns(2).show(ui, |ui| {
        ui.label("ui tick: ");
        ui.label(format!("{}", link.ui_tick));
        ui.end_row();
        ui.label("character: ");
        ui.label(&link.name);
        ui.end_row();
        ui.label("map: ");
        ui.label(format!("{}", link.map_id));
        ui.end_row();
        ui.label("pos: ");
        ui.label(format!("{}, {}", link.window_pos.x, link.window_pos.y));
        ui.end_row();
        ui.label("size: ");
        ui.label(format!("{}, {}", link.window_size.x, link.window_size.y));
        ui.end_row();
    });
}
