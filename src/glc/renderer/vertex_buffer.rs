use std::convert::TryInto;

use glow::*;

pub struct VertexBuffer<'a> {
    pub id: u32,
    gl: &'a glow::Context,
}

impl<'a> VertexBuffer<'a> {
    pub fn new(gl: &'a glow::Context, ) -> VertexBuffer<'a> {
        unsafe {
            let id = gl.create_buffer().expect("failed to create vertex buffer");
            VertexBuffer { id, gl }
        }
    }
    pub fn bind(&self) {
        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.id));
        }
    }
    pub fn unbind(&self) {
        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }
    
}

impl Drop for VertexBuffer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.id);
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct VertexBufferLayoutElement {
    pub etype: u32,
    pub count: i32,
    pub normalized: bool,
}
#[derive(Debug, Default, Clone)]
pub struct VertexBufferLayout {
    layout_of_elements: Vec<VertexBufferLayoutElement>,
}

impl VertexBufferLayout {
    pub fn push_f32(&mut self, count: i32, normalized: bool) {
        &self.layout_of_elements.push(VertexBufferLayoutElement {
            etype: glow::FLOAT,
            count,
            normalized,
        });
    }
    pub fn push_u16(&mut self, count: i32) {
        &self.layout_of_elements.push(VertexBufferLayoutElement {
            etype: glow::UNSIGNED_SHORT,
            count,
            normalized: false,
        });
    }
    pub fn push_u8(&mut self, count: i32) {
        &self.layout_of_elements.push(VertexBufferLayoutElement {
            etype: glow::UNSIGNED_BYTE,
            count,
            normalized: false,
        });
    }
    pub fn push_u32(&mut self, count: i32) {
        &self.layout_of_elements.push(VertexBufferLayoutElement {
            etype: glow::UNSIGNED_INT,
            count,
            normalized: false,
        });
    }

    pub fn set_layout(&self, gl: &glow::Context) {
        let mut stride: i32 = 0;
        for element in self.layout_of_elements.iter() {
            match element.etype {
                FLOAT | UNSIGNED_INT => {
                    stride += 4 * element.count as i32;
                },
                UNSIGNED_BYTE => {
                    stride += 1 * element.count as i32;
                },
                UNSIGNED_SHORT => {
                    stride += 2 * element.count as i32;
                },              
                rest @ _ => {
                    panic!("vertexBufferElement's etype is not right: {}", rest);
                }
            }
        }
        let stride = stride;
        let mut offset = 0;
        for (index, element) in self.layout_of_elements.iter().enumerate() {
            unsafe {
                match element.etype {
                    FLOAT => {
                        gl.vertex_attrib_pointer_f32(
                            index.try_into().unwrap(),
                            element.count,
                            FLOAT,
                            element.normalized,
                            stride,
                            offset,
                        );
                        offset += 4 * element.count as i32;
                    }
                    UNSIGNED_INT => {
                        gl.vertex_attrib_pointer_i32(
                            index.try_into().unwrap(),
                            element.count,
                            UNSIGNED_INT,
                            stride,
                            offset,
                        );
                        offset += 4 * element.count as i32;
                    },
                    UNSIGNED_BYTE => {
                        gl.vertex_attrib_pointer_i32(
                            index.try_into().unwrap(),
                            element.count,
                            UNSIGNED_BYTE,
                            stride,
                            offset,
                        );
                        offset += 1 * element.count as i32;
                    },
                    UNSIGNED_SHORT => {
                        gl.vertex_attrib_pointer_i32(
                            index.try_into().unwrap(),
                            element.count,
                            UNSIGNED_SHORT,
                            stride,
                            offset,
                        );
                        offset += 2 * element.count as i32;
                    },   
                    _ => {
                        panic!("vertexBufferElement's etype is not right");
                    }
                }
                gl.enable_vertex_attrib_array(index.try_into().unwrap());
            }
        }
    }
}
