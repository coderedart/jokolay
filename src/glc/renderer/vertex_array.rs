use glow::HasContext;


pub struct VertexArrayObject<'a> {
    pub id: u32,
    pub gl: &'a glow::Context,
}

impl VertexArrayObject<'_> {
    pub fn new<'a>(gl: &'a glow::Context) -> VertexArrayObject<'a> {
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

impl Drop for VertexArrayObject<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.id);
        }
    }
}
