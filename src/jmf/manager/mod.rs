use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{
    jmf::pack::ZPack,
    joko_render::{JokoRenderer, MarkerQuad},
};
use bitvec::vec::BitVec;
use color_eyre::{eyre::ContextCompat, Result};
use egui::{DragValue, Window};
use indextree::{Arena, NodeId};
use mmarinus::{perms::Read, Private};

use semver::Version;
use serde::{Deserialize, Serialize};

use super::pack::{xml::get_zpack_from_taco, ArchivedZPack};

pub const PACK_LIST_URL: &str = "https://packlist.jokolay.com/packlist.json";
pub struct MarkerManager {
    pub markers_path: PathBuf,
    pub last_update_attempt: f64,
    pub pack_list: poll_promise::Promise<Result<PackList>>,
    pub packs: BTreeMap<String, LivePack>,
    pub packs_being_downloaded: BTreeMap<String, Arc<Mutex<PackDownloadStatus>>>,
    pub ui_data: MarkerManagerUIData,
    pub number_of_markers_to_draw: usize,
}
pub enum PackDownloadStatus {
    Downloading,
    Converting,
    Saving,
    Done,
    Failed(String),
}

#[derive(Debug, Default)]
pub struct MarkerManagerUIData {
    selected_pack: String,
    selected_map: u16,
    selected_marker: usize,
    _selected_trail: usize,
}
pub struct LivePack {
    _mapped: mmarinus::Map<Read, Private>,
    pack: &'static ArchivedZPack,
    pub cattree: Arena<(u16, &'static str)>,
    pub root_category: NodeId,
    // store with rkyv too?
    pub trigger_data: ActivationData,
}

impl LivePack {
    pub fn new(pack_rkyv_path: &Path, _activation_data_path: &Path) -> Result<Self> {
        let mapped = mmarinus::Map::load(pack_rkyv_path, Private, Read)?;

        let pack =
            rkyv::check_archived_root::<ZPack>(&mapped).expect("failed bytecheck for this map");
        let pack: &'static ArchivedZPack = unsafe { std::mem::transmute(pack) };
        let mut cattree = Arena::new();
        let mut nodes = vec![];
        for (cat_index, cat) in pack.cats.iter().copied().enumerate() {
            let n = cattree.new_node((
                cat_index as u16,
                pack.text[cat.display_name as usize].as_str(),
            ));
            nodes.push(n);
            if cat_index != 0 {
                nodes[cat.parent_id as usize].append(n, &mut cattree);
            }
        }
        Ok(Self {
            _mapped: mapped,
            pack,
            cattree,
            root_category: nodes[0],
            trigger_data: ActivationData::default(),
        })
    }
    pub fn get_pack(&self) -> &ArchivedZPack {
        self.pack
    }
    pub fn get_version(&self) -> Version {
        Version::from_str(&self.pack.version).expect("failed ot deserialize version smh")
    }
    pub fn get_timestamp(&self) -> f64 {
        self.pack.timestamp
    }
}
#[derive(Debug, Default)]
pub struct LiveMarkers {}

#[derive(Debug, Default)]
pub struct ActivationData {
    pub enabled_cats: BitVec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PackList {
    pub packs: BTreeMap<String, PackEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackEntry {
    pub version: semver::Version,
    pub url: url::Url,
    pub description: String,
}

impl MarkerManager {
    /// MarkerManager needs the zip files to load as markers and data stored like activation data or enabled categories.
    /// 1.
    pub fn new(markers_path: &Path) -> Result<Self> {
        if !markers_path.is_dir() {
            color_eyre::eyre::bail!("markers path is not a directory");
        }
        let mut packs: BTreeMap<String, LivePack> = Default::default();
        for pack in std::fs::read_dir(markers_path)? {
            let pack = pack?;
            if pack.file_type()?.is_file()
                && pack
                    .path()
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    == "rkyv"
            {
                if let Some(name) = pack.path().file_stem() {
                    let name = name
                        .to_str()
                        .wrap_err("pack name is not valid utf-8")?
                        .to_owned();
                    let live_pack = LivePack::new(pack.path().as_path(), pack.path().as_path())?;
                    packs.insert(name, live_pack);
                }
            }
        }
        let pack_list = poll_promise::Promise::spawn_thread("packlist update thread", || {
            Ok(PackList::default())
        });
        Ok(Self {
            pack_list,
            packs,
            last_update_attempt: 0.0,
            ui_data: Default::default(),
            packs_being_downloaded: BTreeMap::new(),
            markers_path: markers_path.to_owned(),
            number_of_markers_to_draw: 100,
        })
    }

    pub fn load() {}
    pub fn render(&self, map_id: u16, renderer: &mut JokoRenderer) {
        for pack in self.packs.values() {
            if let Some(map_markers) = pack.pack.maps.get(&map_id) {
                for marker in map_markers
                    .markers
                    .iter()
                    .take(self.number_of_markers_to_draw)
                {
                    if renderer.textures.contains_key(marker.texture as u64) {
                        renderer.draw_marker(MarkerQuad {
                            position: marker.position,
                            texture: marker.texture.into(),
                            width: pack.pack.textures[marker.texture as usize].width,
                            height: pack.pack.textures[marker.texture as usize].height,
                        });
                    } else {
                        let png = pack.pack.textures.get(marker.texture as usize).unwrap();
                        let img = image::load_from_memory(&png.bytes).unwrap();
                        let pixels = img.into_rgba8().into_vec();
                        renderer.upload_texture(
                            marker.texture as u32,
                            png.width as u32,
                            png.height as u32,
                            pixels,
                        )
                    }
                }
            }
        }
    }
    pub fn tick(&mut self, etx: &egui::Context, timestamp: f64) {
        Window::new("Marker Manager").show(etx, |ui| {
            ui.add(DragValue::new(&mut self.number_of_markers_to_draw));
            egui::CollapsingHeader::new("Pack List ")
                .default_open(false)
                .show(ui, |ui| {
                    ui.label(format!(
                        "last packlist update attempt: {} seconds ago",
                        (timestamp - self.last_update_attempt) as u64
                    ));
                    if ui.button("update list").clicked() {
                        self.pack_list =
                            poll_promise::Promise::spawn_thread("packlist update thread", || {
                                let packlist: PackList =
                                    ureq::get(PACK_LIST_URL).call()?.into_json()?;
                                Ok(packlist)
                            });
                        self.last_update_attempt = timestamp;
                    }

                    match self.pack_list.ready() {
                        Some(plist) => match plist {
                            Ok(plist) => {
                                for (pack_name, pack_entry) in plist.packs.iter() {
                                    ui.group(|ui| {
                                        if self.packs.contains_key(pack_name) {
                                            ui.label("status: Installed");
                                        } else if ui.button("Install").clicked() {
                                            let status = Arc::new(Mutex::new(
                                                PackDownloadStatus::Downloading,
                                            ));
                                            let url = pack_entry.url.clone();
                                            let version = pack_entry.version.clone();
                                            let name = pack_name.clone() + ".rkyv";
                                            let markers_path = self.markers_path.clone();
                                            self.packs_being_downloaded
                                                .insert(pack_name.clone(), status.clone());
                                            std::thread::spawn(move || {
                                                let xmlpack =
                                                    match ureq::get(url.as_str()).call() {
                                                        Ok(response) => {
                                                            dbg!(&response);
                                                            let mut v = vec![];
                                                            match response
                                                                .into_reader()
                                                                .read_to_end(&mut v)
                                                            {
                                                                Ok(len) => {
                                                                    dbg!(len);
                                                                    v
                                                                }
                                                                Err(e) => {
                                                                    *status.lock().unwrap() =
                                                            PackDownloadStatus::Failed(
                                                                e.to_string(),
                                                            );
                                                                    return;
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            *status.lock().unwrap() =
                                                                PackDownloadStatus::Failed(
                                                                    e.to_string(),
                                                                );
                                                            return;
                                                        }
                                                    };
                                                match get_zpack_from_taco(&xmlpack, version) {
                                                    Ok((zpack, _, failures)) => {
                                                        dbg!(failures);
                                                        let zkyv = rkyv::to_bytes::<_, 100000>(&zpack).expect("failed to serialize data");
                                                        std::fs::write(markers_path.join(name), &zkyv).expect("failed to write to tekkit rkyv");
                                                    },
                                                    Err(e) => {
                                                        *status.lock().unwrap() =
                                                        PackDownloadStatus::Failed(
                                                            e.to_string(),
                                                        );
                                                    },
                                                }
                                            });
                                        }
                                        ui.horizontal(|ui| {
                                            ui.label("id: ");
                                            ui.label(pack_name.as_str());
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("description: ");
                                            ui.label(pack_entry.description.as_str());
                                        });

                                        ui.label(format!("version: {}", pack_entry.version));

                                        ui.horizontal(|ui| {
                                            ui.label("url: ");
                                            ui.label(pack_entry.url.as_str());
                                        });
                                    });
                                }
                            }
                            Err(e) => {
                                ui.label(format!("failed to get packlist. error: {e}"));
                            }
                        },
                        None => {
                            ui.label("pack list still pending");
                        }
                    }
            });
            egui::CollapsingHeader::new("Loaded Packs")
                .default_open(false)
                .show(ui, |ui| {
                    ui.columns(4, |columns| {
                        match columns {
                            [c0, c1, c2, c3] => {
                                for (pack_name, zpack) in self.packs.iter() {
                                    if c0
                                        .selectable_label(
                                            self.ui_data.selected_pack.as_str() == pack_name,
                                            pack_name,
                                        )
                                        .clicked()
                                    {
                                        if self.ui_data.selected_pack.as_str() == pack_name {
                                            self.ui_data.selected_pack.clear();
                                        } else {
                                            self.ui_data.selected_pack = pack_name.to_string();
                                        }
                                    }
                                    if &self.ui_data.selected_pack == pack_name {
                                        let height = c1.text_style_height(&egui::TextStyle::Body);
                                        egui::ScrollArea::new([false, true])
                                            .id_source("pack scroll area")
                                            .show_rows(
                                            c1, height,
                                        zpack.pack.maps.len(), |ui, range| {
                                            for (&map, mapdata) in zpack.pack.maps.iter().skip(range.start).take(range.end  - range.start) {
                                                if ui.selectable_label(self.ui_data.selected_map == map, format!("{map}")).clicked() {
                                                    if self.ui_data.selected_map == map {
                                                        self.ui_data.selected_map = 0;
                                                    } else {
                                                        self.ui_data.selected_map = map;
                                                    }
                                                }
                                                if map == self.ui_data.selected_map {
                                                    c2.horizontal(|ui| {
                                                        ui.label("total markers: ");
                                                        ui.label(&format!("{}", mapdata.markers.len()));
                                                    });
                                                    let height = c2.text_style_height(&egui::TextStyle::Body);
                                                    egui::ScrollArea::new([false, true])
                                                    .id_source("map scroll area")
                                                    .show_rows(c2, height,
                                                    mapdata.markers.len(), |ui, range| {
                                                        for (index,  marker) in mapdata.markers[range.clone()].iter().enumerate() {
                                                            let index = range.start + index;
                                                            if ui.selectable_label(self.ui_data.selected_marker == index, format!("{index}")).clicked() {
                                                                if self.ui_data.selected_marker == index {
                                                                    self.ui_data.selected_marker = 0;
                                                                } else {
                                                                    self.ui_data.selected_marker = index;
                                                                }
                                                            }
                                                            if self.ui_data.selected_marker == index {
                                                                c3.label(format!("pos: x = {}; y = {}; z = {}", marker.position.x, marker.position.y, marker.position.z));
                                                            }
                                                        }
                                                    });
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                            _ => panic!("whatever")
                        }
                    });
            });
        });
    }
}
