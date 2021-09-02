use std::time::Instant;

use uuid::Uuid;

use crate::core::painter::opengl::buffer::{Buffer, VertexBufferLayout, VertexBufferLayoutTrait};


/// Trail Vertex contains the vertex position in world space
#[derive(Debug, Clone, Copy, Default)]
pub struct TrailVertex {
    pub vpos: [f32; 3],
    pub tex_coords: [f32; 3],
}

impl VertexBufferLayoutTrait for TrailVertex {
    fn get_layout() -> VertexBufferLayout {
        let mut layout = VertexBufferLayout::default();
        layout.push_f32(3, false);
        layout.push_f32(3, false);
        layout
    }
}

unsafe impl bytemuck::Zeroable for TrailVertex {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for TrailVertex {}


/// A struct for holding a gpu buffer which contains the mesh of vertices that make up a trail
pub struct TrailMeshBuffer {
    /// Uuid of the trail for identification
    pub id: Uuid,
    /// the instant when trail mesh was last modified. if it was modified again, we compare it to this, and update if they are not equal.
    pub modified: Instant,
    /// the gpu buffer which contains the vertices from the mesh that matched the above properties. only update IF the above don't match. and dropping the struct will delete the buffer in main thread
    pub vb: Buffer
}