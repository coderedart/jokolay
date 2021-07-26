use std::rc::Rc;

use glow::{HasContext, NativeVertexArray};

pub struct VertexArrayObject {
    pub id: NativeVertexArray,
    pub gl: Rc<glow::Context>,
}

impl VertexArrayObject {
    pub fn new(gl: Rc<glow::Context>) -> VertexArrayObject {
        unsafe {
            let id = gl.create_vertex_array().unwrap();
            VertexArrayObject { id, gl }
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.id));
        }
    }
    pub fn unbind(&self) {
        unsafe {
            self.gl.bind_vertex_array(None);
        }
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.id);
        }
    }
}
