use crate::gw::marker::Marker;

use super::vertex_buffer::VertexBufferLayout;

#[derive(Debug,  Clone, Copy)]
pub struct Node {
    pub position: [f32; 3],
    pub scale: f32,
    pub alpha: f32,
    pub height_offset: f32,
    pub fade_near: u32,
    pub fade_far: u32,
    pub min_size: u32,
    pub max_size: u32,
    pub color: [u8; 4],
    pub tex_layer: u16,
    pub tex_slot: u16,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            position: [0.0,0.0,0.0],
            scale: 1.0,
            alpha: 1.0,
            height_offset: 1.5,
            fade_near: 0,
            fade_far: u32::MAX,
            min_size: 0,
            max_size: u32::MAX,
            color: [0,0,0,0],
            tex_layer: 0,
            tex_slot: 0,
        }
    }
}

impl From<&Marker> for Node {
    fn from(m: &Marker) -> Self {
        let mut n = Node::default();
        n.position = [m.xpos, m.ypos, m.zpos];
        if let Some(p) = m.map_display_size {
            n.scale = p as f32;
        }
        if let Some(p) = m.alpha {
            n.alpha = p;
        }
        if let Some(p) = m.height_offset {
            n.height_offset = p;
        }
        if let Some(p) = m.fade_near {
            n.fade_near = p ;
        }
        if let Some(p) = m.fade_far {
            n.fade_far = p ;
        }
        if let Some(p) = m.min_size {
            n.min_size = p ;
        }
        if let Some(p) = m.max_size {
            n.max_size = p ;
        }
        if let Some(p) = m.color {
            n.color = p.to_ne_bytes();
        }
        

        n
    }
}



impl Node {
    pub fn get_buffer_layout() -> VertexBufferLayout {
        let mut vbl = VertexBufferLayout::default();
        vbl.push_f32(3, false);
        vbl.push_f32(1, false);
        vbl.push_f32(1, false);
        vbl.push_f32(1, false);
        vbl.push_u32(1);
        vbl.push_u32(1);
        vbl.push_u32(1);
        vbl.push_u32(1);
        vbl.push_u8(4);
        vbl.push_u16(1);
        vbl.push_u16(1);
        vbl
    }
    
}












unsafe impl bytemuck::Zeroable for Node {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for Node {}

/*
vertex_position,
tex_id,
tex_coords,

*/
