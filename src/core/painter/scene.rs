use std::{collections::HashMap, time::Instant};

use egui::{ClippedMesh, Rect};





/// This struct will store the data needed to render. when there's Some(data), it means we want to update the gl buffers. 
/// when its None, it means there's no update and should continue as usual. and when there's Some(Vec::new()) like emptry data, it means we 
/// want to clear the buffers and not draw that.
pub struct Scene {
    pub egui_meshes: Option<Vec<ClippedMesh>>,
    pub marker_2d_static: Instant,
    pub marker_2d_dynamic: Instant,
    pub trail_static: Instant,
    pub trail_dynamic: Instant,
    pub marker_3d_static: Instant,
    pub marker_3d_dynamic: Instant,
    pub texture_locations: Option<HashMap<usize, (u32, Rect)>>,
    
}

