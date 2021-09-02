use crate::{core::fm::{FileManager, RID}, tactical::{
        localtypes::category::{CategoryIndex, IMCategory},
        xmltypes::xml_trail::XMLTrail,
    }};
use std::{collections::HashMap, io::Read};
use uuid::Uuid;

/// The trail data struct holding the nodes
#[derive(Debug, Clone)]
pub struct TrailData {
    pub map_id: u32,
    pub version: u32,
    pub nodes: Vec<[f32; 3]>,
}
impl TrailData {
    pub fn parse_from_file(path: RID, fm: &FileManager) -> Option<TrailData> {
        let tfile = fm
            .get_path(path)
            .unwrap()
            .open_file()
            .map_err(|e| {
                log::error!("unable to open trail data file. error: {:?}", &e);
                e
            })
            .unwrap();
        let mut reader = std::io::BufReader::new(tfile);
        let mut buffer = [0_u8; 4];
        reader.read_exact(&mut buffer).unwrap();
        let version = u32::from_ne_bytes(buffer);

        reader.read_exact(&mut buffer).unwrap();
        let map_id = u32::from_ne_bytes(buffer);

        let mut nodes = vec![];
        reader.read_to_end(&mut nodes).unwrap();
        let nodes: Vec<f32> = bytemuck::cast_slice(&nodes).to_vec();
        let nodes: Vec<[f32; 3]> = nodes
            .chunks_exact(3)
            .map(|pos| [pos[0], pos[1], pos[2]])
            .collect();
        Some(TrailData {
            map_id,
            version,
            nodes,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Trail {
    pub category: CategoryIndex,
    pub guid: Uuid,
    pub trail_data_file: RID,
    pub texture: Option<RID>,
    pub anim_speed: Option<f32>,
    pub trail_scale: Option<f32>,
    pub color: Option<[u8; 4]>,
    pub alpha: Option<f32>,
    pub fade_near: Option<u32>,
    pub fade_far: Option<u32>,
    pub tdata: TrailData,
}

impl Trail {
    pub fn from_xml_trail(
        pack_path: RID,
        trail: &XMLTrail,
        global_cats: &Vec<IMCategory>,
        fm: &FileManager,
    ) -> Option<Self> {
        let category = global_cats
            .iter()
            .position(|c| c.full_name == trail.category)
            .or_else(|| {
                log::error!(
                    "could not find category {:?} for the trail {:?}",
                    trail.category,
                    trail.guid,
                );
                None
            })?;

        let category = CategoryIndex(category);
        let icon_path = trail.texture.clone();
        let icon_vid = if let Some(ipath) = icon_path {
            let pack_path = fm.get_path(pack_path).unwrap();
            let ipath = pack_path.join(&ipath).unwrap();
            if let Some(v) = fm.get_vid(&ipath) {
                Some(v)
            } else {
                log::error!(
                    "could not get texture path for trail: {:?}, {:?}, {:?}, {:?}, {:?}",
                    ipath,
                    pack_path,
                    trail.guid,
                    &trail.texture,
                    &trail.category
                );
                None
            }
        } else {
            None
        };
        let trail_file = trail.trail_data_file.clone();
        let pack_path = fm.get_path(pack_path).unwrap();
        let tpath = pack_path.join(&trail_file).unwrap();
        let trail_data_file = if let Some(v) = fm.get_vid(&tpath) {
            v
        } else {
            log::error!(
                "cannot find trail data file. trail path: {}, pack_path: {}, trail_guid:{:?}, traildata tag: {}, trail category: {}",
                tpath.as_str(),
                pack_path.as_str(),
                trail.guid,
                &trail.trail_data_file,
                &trail.category
            );
            return None;
        };
        let tdata = TrailData::parse_from_file(trail_data_file, fm).unwrap();
        Some(Trail {
            category,
            guid: trail.guid.unwrap(),
            trail_data_file,
            texture: icon_vid,
            anim_speed: trail.anim_speed,
            trail_scale: trail.trail_scale,
            color: trail.color,
            alpha: trail.alpha,
            fade_near: trail.fade_near,
            fade_far: trail.fade_far,
            tdata,
        })
    }
    /// turns a Vec<Trail> into Vec<Uuid> by inserting the trail into a `all_trails` hashmap to keep all trails in one place to avoid duplication. as well as maintain the order by using a Vec<Uuid>
    pub fn get_vec_uuid_trail(
        trails: Vec<XMLTrail>,
        all_trails: &mut HashMap<Uuid, Trail>,
        pack_path: RID,
        global_cats: &mut Vec<IMCategory>,
        fm: &FileManager,
    ) -> Vec<Uuid> {
        let mut result = Vec::new();
        for xt in trails {
            let id = xt.guid.unwrap();
            if let Some(t) = Trail::from_xml_trail(pack_path, &xt, global_cats, fm) {
                all_trails.entry(id).or_insert(t);
                result.push(id);
            } else {
                log::error!(
                    "could not get Trail from xml_trail. {:?} {:?} {:?}",
                    &xt.guid,
                    pack_path,
                    &xt.trail_data_file
                )
            }
        }
        result
    }
}

impl Trail {
    pub fn register_category(&self, global_cats: &mut Vec<IMCategory>) {
        if let Some(c) = global_cats.get_mut(self.category.0) {
            c.trail_registry.push(self.guid);
        } else {
            log::error!(
                "marker with guid: {:?} cannot find category: {:?} to register",
                self.guid,
                self.category
            );
        }
    }
}
