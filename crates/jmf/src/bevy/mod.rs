use std::{collections::BTreeMap, path::PathBuf};

use bevy::{asset::AssetServerSettings, prelude::*};
use walkdir::WalkDir;

use crate::manager::pack::Pack;

pub struct MarkerPlugin;

impl Plugin for MarkerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let assets_path = app
            .world
            .get_resource::<AssetServerSettings>()
            .expect("failed to get asset server settings")
            .asset_folder
            .clone();
        let assets_path = assets_path.into();
        app.insert_resource(MarkerManager::new(assets_path));
    }
}

pub struct LivePack {
    pub pack: Pack,
    // pub activation_data: ActivationData,
    // character name to live marker entity ids along with their index in the current map markers.
    // despawn them to avoid memory leaks
    // pub live_markers: BTreeMap<String, BTreeMap<usize, Entity>>,
    // pub live_trails: BTreeMap<String, BTreeMap<usize, Entity>>
}

pub struct MarkerManager {
    pub pack_dir: PathBuf,
    pub packs: BTreeMap<String, Pack>,
}
impl MarkerManager {
    fn new(mut assets_path: PathBuf) -> Self {
        assets_path.push("data");
        assets_path.push("packs");
        std::fs::create_dir_all(&assets_path).expect("failed to create assets directory");
        let mut packs: BTreeMap<String, Pack> = Default::default();
        for pack_entry in WalkDir::new(&assets_path).max_depth(1).min_depth(1) {
            let pack_entry = pack_entry.expect("failed to entry pack");
            assert!(pack_entry
                .metadata()
                .expect("failed to get metadata")
                .is_dir());
            let pack = match Pack::load_from_directory(&pack_entry.path()) {
                Ok(p) => p,
                Err(e) => {
                    let _ = dbg!(e);
                    panic!("failed ot deserialize pack from directory")
                }
            };
            packs.insert(
                pack_entry
                    .file_name()
                    .to_str()
                    .expect("failed to get name for pack dir")
                    .to_string(),
                pack,
            );
        }
        Self {
            pack_dir: assets_path,
            packs,
        }
    }
}
/*
startup system to load marker packs
system to check current live maps and adjust the pack's live map + character
system to spawn marker objects for rendering (and save them)
system to deal with "sensing" markers that want to trigger based on distance + action key
system to edit markers and save them
system to install markers

*/

// pub struct JokolayClient(usize);
// pub struct Locks {
//     pub markers: BTreeMap<UniqueMarkerID, JokolayClient>,
//     pub maps: BTreeMap<u16, JokolayClient>,
//     pub category_menu: Option<JokolayClient>
// }
