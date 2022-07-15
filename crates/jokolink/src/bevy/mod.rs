use std::sync::Arc;

use bevy::{
    core::{FixedTimestep, Time},
    prelude::{
        info, App, CoreStage, EventReader, EventWriter, ParallelSystemDescriptorCoercion, Plugin,
        Res, ResMut,
    },
    window::{CreateWindow, WindowDescriptor, WindowId, Windows},
};
use bevy_egui::EguiContext;
use indexmap::IndexMap;
use x11rb::rust_connection::RustConnection;

use crate::{
    mlink::MumbleLink,
    mumble_file::{GW2InstanceData, MumbleFile, MumbleFileTrait},
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
        #[cfg(target_family = "unix")]
        app.insert_resource(
            RustConnection::connect(None)
                .expect("failed to create Rust Connection")
                .0,
        );
        app.insert_resource(GW2InstanceCollection::default());
        app.insert_resource(MumbleConfig::default());
        app.add_startup_system(initialize_mumble_files);
        app.add_system_to_stage(CoreStage::First, mumble_creator);
        app.add_system_to_stage(CoreStage::First, mumble_updater.after(mumble_creator));
        app.add_system_to_stage(CoreStage::First, mumble_destroyer.after(mumble_updater));
        app.add_system(mumble_window_show);
        app.add_system(register_live_gw2_instances);
        app.add_system(
            match_window_gw2_instance.with_run_criteria(FixedTimestep::steps_per_second(0.5)),
        );
    }
}

fn initialize_mumble_files(
    config: Res<MumbleConfig>,
    mut mumble_creator: EventWriter<CreateMumbleFile>,
) {
    for key in config.links.iter() {
        mumble_creator.send(CreateMumbleFile(key.clone()));
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
/// This deals primarily with how jokolay will "attach" itself to a running Guild Wars 2 Game Instance.
/// The defaults will consider the single-boxing to be the main use case and support that.
pub struct MumbleConfig {
    /// The Mumble link names that we should be creating MumbleFiles for and monitoring them for activity
    /// the links that come first are said to have "higher" priority than links that come later. see `reattach_main_priority` field docs below to understand what i mean
    /// example: `links : ["MumbleLink", "AltAccount"]`
    /// default is just `["MumbleLink"]` for monitoring one gw2 instance, but if multiple gw2 instances use the same link name, it can be confusing.
    pub links: Vec<String>,
    /// whether main window should attach itself to an active gw2 instance
    /// `true` : attach to the first active gw2 instance and if that instance quits, stay unattached until next instance starts and attach to that.
    /// `false`: stay unattached and independent. useful for when you want jokolay to stay where it is and not on the main guild wars 2 window.
    /// default is `true`. when gw2 launches, we should want to attach to it.
    pub main_window: bool,
    /// if main window is currently attached a gw2 instance. but we see a new instance from a higher priority link, should main_window automatically switch
    /// to that instance?
    /// `true` : deattach the current instance and switch the the higher priority instance.
    /// `false`: nope, just stay attached to the current link. users can manually detach / attach if necessary.
    /// default is `false`, because we only consider one gw2 instance by default. we don't expect another to exist at all.
    pub reattach_main_priority: bool,
    /// whether we should create secondary windows when we find more than one gw2 instance being active at a time.
    /// `true` : we will create and close secondary windows depending on the number of active gw2 instances
    /// `false`: we will just let the gw2 instances be without any window.
    /// default is `false`. we will stick with just primary window and won't bother with multiple gw2 instances.
    pub secondary_windows: bool,
}
impl Default for MumbleConfig {
    fn default() -> Self {
        Self {
            links: vec!["MumbleLink".to_string()],
            main_window: true,
            reattach_main_priority: Default::default(),
            secondary_windows: Default::default(),
        }
    }
}
pub struct GW2InstanceWindow {
    pub unique_id: u32,
    pub data: GW2InstanceData,
    pub associated_window: Option<WindowId>,
}

#[derive(Default)]
pub struct GW2InstanceCollection {
    pub attached_window_pairs: IndexMap<u32, GW2InstanceWindow>,
}

fn register_live_gw2_instances(
    mut mumble_update_events: EventReader<MumbleUpdate>,
    mut gw2_instance_collection: ResMut<GW2InstanceCollection>,
    windows: Res<Windows>,
    mut window_creator: EventWriter<CreateWindow>,
    config: ResMut<MumbleConfig>,
    #[cfg(target_family = "unix")] xc: Res<RustConnection>,
) {
    for mumble_update in mumble_update_events.iter() {
        let unique_id = mumble_update.unique_id;
        if !gw2_instance_collection
            .attached_window_pairs
            .contains_key(&unique_id)
        {
            info!(
                "found new gw2 instance with unique_id: {} and link_name: {}",
                unique_id, &mumble_update.key
            );

            #[cfg(target_family = "unix")]
            let data = GW2InstanceData::new(unique_id as usize, &xc)
                .expect("failed to create gw2 instance data from unique_id and xc");

            #[cfg(target_os = "windows")]
            let data = todo!();

            let mut instance_window = GW2InstanceWindow {
                unique_id,
                data,
                associated_window: None,
            };
            // if autoattach main window or create secondary window is true, we need to search for a window to attach to the new instance
            if config.main_window || config.secondary_windows {
                // go through all the windows and see if there's a window that is not used yet.
                let free_window = windows.iter().find_map(|window| {
                    let window_id = window.id();
                    // check whether any of the existing instances use the window
                    let window_used = gw2_instance_collection.attached_window_pairs.iter().any(
                        |(_, instance_window)| {
                            instance_window
                                .associated_window
                                .map(|id| id == window_id) // if associated window exists, check if its the same id as this window
                                .unwrap_or_default() // if it doesn't exist, we will return false (default)
                        },
                    );
                    if window_used {
                        None
                    } else {
                        Some(window)
                    }
                });
                if let Some(window) = free_window {
                    // if there's a free window
                    if window.id().is_primary() {
                        // and the free window is the primary window
                        if config.main_window {
                            // AND attach main window option is true
                            instance_window.associated_window = Some(window.id());
                        }
                    } else {
                        instance_window.associated_window = Some(window.id()); // if its a secondary window, just attach it
                    }
                } else {
                    // if there's no free window
                    if config.secondary_windows {
                        // if we are allowed to create secondary windows
                        window_creator.send(CreateWindow {
                            id: WindowId::new(),
                            descriptor: WindowDescriptor {
                                width: 800.0,
                                height: 600.0,
                                title: format!("Link {}", &mumble_update.key),
                                decorations: false,
                                transparent: true,
                                ..Default::default()
                            },
                        });
                        // skip adding the instance, so that we check for a free window next frame and use this freshly created secondary window :)
                        return;
                    }
                }
            }
            gw2_instance_collection
                .attached_window_pairs
                .insert(unique_id, instance_window);
        }
    }
}

fn match_window_gw2_instance(
    gw2_instance_collection: ResMut<GW2InstanceCollection>,
    mut windows: ResMut<Windows>,
    #[cfg(target_family = "unix")] xc: Res<RustConnection>,
) {
    for instance in gw2_instance_collection.attached_window_pairs.values() {
        if let Some(window) = instance.associated_window.map(|id| {
            windows
                .get_mut(id)
                .expect("failed to find window with this id")
        }) {
            #[cfg(target_family = "unix")]
            let dimensions = instance
                .data
                .get_window_dimensions(&xc)
                .expect("failed to get window dimensions");
            info!("dimensions: {:#?}", &dimensions);
            let pos = window.position().expect("failed to get window position");
            let (width, height) = (window.width(), window.height());
            info!(
                "bevy window dimensions: x {} y {} width {} height {}",
                pos.x, pos.y, width, height
            );
            if dimensions.x != pos.x
                || dimensions.y != pos.y
                || dimensions.width != width as u32
                || dimensions.height != height as u32
            {
                info!("resizing bevy window to fit the position / size of gw2 instance window");
                window.set_position([dimensions.x, dimensions.y].into());
                window.set_resolution(dimensions.width as f32, dimensions.height as f32);
            }
        }
    }
}
