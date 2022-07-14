use std::sync::Arc;

use bevy::{
    core::Time,
    prelude::{
        App, CoreStage, EventReader, EventWriter, ParallelSystemDescriptorCoercion, Plugin, Res,
        ResMut,
    },
};
use bevy_egui::EguiContext;
use indexmap::IndexMap;

use crate::{
    mlink::MumbleLink,
    mumble_file::{MumbleFile, MumbleFileTrait},
};

/// This event tells MumblePlugin to create a MumbleLink File with the provided string as link name.
/// MumblePlugin will create the file and start reading data from it every frame.
pub struct CreateMumbleFile(String);
pub struct DeleteMumbleFile(String);

pub struct MumbleUpdate {
    pub key: Arc<str>,
    pub unique_id: u32,
    pub link: Arc<MumbleLink>,
}

type MumbleFileCollection = IndexMap<Arc<str>, MumbleFile>;
pub struct MumblePlugin;

impl Plugin for MumblePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateMumbleFile>()
            .add_event::<DeleteMumbleFile>()
            .add_event::<MumbleUpdate>();
        app.insert_resource(MumbleFileCollection::default());
        app.insert_resource(ShowMumbleWindow(true));
        app.add_system_to_stage(CoreStage::First, mumble_creator);
        app.add_system_to_stage(CoreStage::First, mumble_updater.after(mumble_creator));
        app.add_system_to_stage(CoreStage::First, mumble_destroyer.after(mumble_updater));
        app.add_system(mumble_window_show);
    }
}

fn mumble_creator(
    time: Res<Time>,
    mut mumble_collection: ResMut<MumbleFileCollection>,
    mut create_events: EventReader<CreateMumbleFile>,
) {
    for create_event in create_events.iter() {
        bevy::log::info!(
            "mumble file creation event for key : {}",
            create_event.0.as_str()
        );
        mumble_collection
            .entry(create_event.0.as_str().into())
            .or_insert_with(|| {
                MumbleFile::new(&create_event.0, time.seconds_since_startup())
                    .expect("failed to create mumble file")
            });
    }
}

fn mumble_destroyer(
    mut mumble_collection: ResMut<MumbleFileCollection>,
    mut create_events: EventReader<DeleteMumbleFile>,
) {
    for delete_event in create_events.iter() {
        bevy::log::info!(
            "mumble file destroy event for key : {}",
            delete_event.0.as_str()
        );
        mumble_collection.remove(delete_event.0.as_str());
    }
}

fn mumble_updater(
    time: Res<Time>,
    mut mumble_collection: ResMut<MumbleFileCollection>,
    mut update_events: EventWriter<MumbleUpdate>,
) {
    for (key, file) in mumble_collection.iter_mut() {
        let data = file
            .get_link(time.seconds_since_startup())
            .expect("failed to get link updated data");
        if let Some(umd) = data {
            update_events.send(MumbleUpdate {
                key: key.clone(),
                unique_id: umd.unique_id,
                link: Arc::new(umd.link),
            });
        }
    }
}

pub struct ShowMumbleWindow(bool);

fn mumble_window_show(
    mut show: ResMut<ShowMumbleWindow>,
    mut egui_ctx: ResMut<EguiContext>,
    mumble_collection: ResMut<MumbleFileCollection>,
) {
    bevy_egui::egui::Window::new("Mumble Files")
        .open(&mut show.0)
        .show(&egui_ctx.ctx_mut(), |ui| {
            for (key, link) in mumble_collection.iter() {
                ui.horizontal(|ui| {
                    ui.label(key.as_ref());
                    ui.label(format!("{:#?}", link.get_previous_tick()));
                });
            }
        });
}

/// The configuration used by MumblePlugin.
pub struct MumbleConfig {
    /// The Mumble link names that we should be creating MumbleFiles for and monitoring them for activity
    /// the links that come first are said to have "higher" priority than links that come later. see `reattach_main_priority` field docs below to understand what i mean
    /// example: `links : ["MumbleLink", "AltAccount"]`
    pub links: Vec<String>,
    /// whether main window should attach itself to an active gw2 instance
    /// `true` : attach to the first active gw2 instance and if that instance quits, stay unattached until next instance starts and attach to that.
    /// `false`: stay unattached and independent. useful for when you want jokolay to stay where it is and not on the main guild wars 2 window.
    pub main_window: bool,
    /// if main window is currently attached a gw2 instance. but we see a new instance from a higher priority link, should main_window automatically switch
    /// to that instance?
    /// `true` : deattach the current instance and switch the the higher priority instance.
    /// `false`: nope, just stay attached to the current link. users can manually detach / attach if necessary.
    pub reattach_main_priority: bool,
    pub secondary_windows: bool,
}

pub struct GW2InstanceCollection {}
