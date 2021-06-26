use egui::epaint::Vertex;

use super::renderer::vertex_buffer::VertexBufferLayout;





    pub fn get_egui_vertex_buffer_layout() -> VertexBufferLayout {
        let mut vbl = VertexBufferLayout::default();
        vbl.push_f32(2, false);
        vbl.push_f32(2, false);
        vbl.push_u32(1);
        vbl
    }
    
