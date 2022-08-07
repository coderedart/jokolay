use bevy::{prelude::*};
use bevy_egui::EguiContext;
use jokolink::{
    mlink::MumbleLink,
    mumble_file::{GW2InstanceData, MumbleFile, MumbleFileTrait},
};

pub struct MumblePlugin;

impl bevy::app::Plugin for MumblePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // need x11 connection to get pid from xid for gw2 window. then, we can check if gw2 is alive by using the pid
        #[cfg(target_family = "unix")]
        app.insert_resource(
            jokolink::mumble_file::linux::RustConnection::connect(None)
                .expect("failed to crate rust conn")
                .0,
        );
        let mfile = MumbleFile::new("MumbleLink", 0.0).expect("failed to crate mumble file");

        app.insert_resource(JokoLinkData {
            mfile,
            instance: None,
            last_resize_check: 0.0,
        });
        app.insert_resource(MumbleLink::default());
        app.add_system_to_stage(CoreStage::First, jokolink_tick);
        app.add_system(mumble_window);
    }
}
struct JokoLinkData {
    pub mfile: MumbleFile,
    pub instance: Option<GW2InstanceData>,
    pub last_resize_check: f64,
}

fn jokolink_tick(
    mut jldata: ResMut<JokoLinkData>,
    mut mumble_link: ResMut<MumbleLink>,
    dt: Res<Time>,
    mut windows: ResMut<Windows>,
    #[cfg(target_family = "unix")] xc: Res<jokolink::mumble_file::linux::RustConnection>,
) {
    let latest_time = dt.time_since_startup().as_secs_f64();
    match jldata.mfile.get_link(latest_time) {
        Ok(updated_mumble_data) => match updated_mumble_data {
            Some(mdata) => {
                match jldata.instance.as_ref().map(|i| i.get_unique_id()) {
                    Some(unique_id) => {
                        if unique_id != mdata.unique_id {
                            jldata.instance = Some(
                                GW2InstanceData::new(mdata.unique_id as usize, &xc)
                                    .expect("failed to get gw2 instance from unique id"),
                            );
                        }
                    }
                    None => {
                        jldata.instance = Some(
                            GW2InstanceData::new(mdata.unique_id as usize, &xc)
                                .expect("failed to get gw2 instance from unique id"),
                        );
                    }
                }

                *mumble_link = mdata.link;
            }
            None => {}
        },
        Err(e) => {
            panic!("{e}");
        }
    }

    // resize/  reposition to match gw2
    if dt.time_since_startup().as_secs_f64() - jldata.last_resize_check > 5.0 {
        jldata.last_resize_check = dt.time_since_startup().as_secs_f64();
        if let Some(instance_data) = jldata.instance.as_mut() {
            let window = windows.primary_mut();
            #[cfg(target_family = "unix")]
            let dimensions = instance_data
                .get_window_dimensions(&xc)
                .expect("failed to get window dimensions");
            let pos = window.position().expect("failed to get window position");
            let (width, height) = (window.width(), window.height());

            if dimensions.x != pos.x
                || dimensions.y != pos.y
                || dimensions.width != width as u32
                || dimensions.height != height as u32
            {
                info!("dimensions: {:#?}", &dimensions);
                info!(
                    "bevy window dimensions: x {} y {} width {} height {}",
                    pos.x, pos.y, width, height
                );
                info!("resizing bevy window to fit the position / size of gw2 instance window");
                window.set_position([dimensions.x, dimensions.y].into());
                window.set_resolution(dimensions.width as f32, dimensions.height as f32);
            }
        }
    }
}

fn mumble_window(
    _jldata: Res<JokoLinkData>,
    _mlink: Res<MumbleLink>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    bevy_egui::egui::Window::new("Mumble Window").show(egui_ctx.ctx_mut(), |ui| {
        ui.label("hello mumble");
    });
}
