use crate::{client::{am::AssetManager, tactical::{
        localtypes::category::{CategoryIndex, IMCategory},
        xmltypes::xml_trail::XMLTrail,
    }}};
use std::{io::Read};
use anyhow::Context;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tokio::io::AsyncReadExt;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Default, Deserialize)]
pub struct TrailIndex(pub usize);

/// The trail data struct holding the nodes
#[derive(Debug, Clone)]
pub struct TrailData {
    pub version: u32,
    pub map_id: u32,
    pub nodes: Vec<[f32; 3]>,
}
impl TrailData {
    pub async fn parse_from_file(path: usize, am: &AssetManager) -> anyhow::Result<TrailData> {
        let mut tfile = am.open_file(path).await?;
        let mut buffer = [0_u8; 4];
        tfile.read_exact(&mut buffer).await?;
        let version = u32::from_ne_bytes(buffer);

        tfile.read_exact(&mut buffer).await;
        let map_id = u32::from_ne_bytes(buffer);

        let mut nodes = vec![];
        tfile.read_to_end(&mut nodes).await?;
        let nodes: Vec<f32> = bytemuck::cast_slice(&nodes).to_vec();
        let nodes: Vec<[f32; 3]> = nodes
            .chunks_exact(3)
            .map(|pos| [pos[0], pos[1], pos[2]])
            .collect();
        Ok(TrailData {
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
    pub trail_data_file: usize,
    pub texture: Option<usize>,
    pub anim_speed: Option<f32>,
    pub trail_scale: Option<f32>,
    pub color: Option<[u8; 4]>,
    pub alpha: Option<f32>,
    pub fade_near: Option<u32>,
    pub fade_far: Option<u32>,
    pub tdata: TrailData,
}

impl Trail {
    pub async fn from_xml_trail(
        pack_path_id: usize,
        trail: &XMLTrail,
        global_cats: &Vec<IMCategory>,
        am: &AssetManager,
    ) -> anyhow::Result<Self> {
        let category = global_cats
            .iter()
            .position(|c| c.full_name == trail.category)
            .context(format!(
                    "could not find category {:?} for the trail {:?}",
                    trail.category,
                    trail.guid,
                ))?;

        let category = CategoryIndex(category);
        let icon_vid = if let Some(ref ipath) = trail.texture {
            let ipath = am.pack_relative_to_absolute_path(pack_path_id, ipath)?;
                Some(am.get_id_from_file_path(&ipath)?)
        } else {
            None
        };
        let trail_data_path = am.pack_relative_to_absolute_path(pack_path_id, &trail.trail_data_file)?;
        let trail_data_file = am.get_id_from_file_path(&trail_data_path).context(format!(
                "cannot find trail data file. trail path: {:?}, trail_guid:{:?}, traildata tag: {}, trail category: {}",
                trail_data_path,
                trail.guid,
                &trail.trail_data_file,
                &trail.category
            ))?;
        let tdata = TrailData::parse_from_file(trail_data_file, am).await?;
        Ok(Trail {
            category,
            guid: trail.guid.unwrap_or(Uuid::new_v4()),
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
