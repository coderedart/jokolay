use glow::*;
use std::rc::Rc;

use crate::gl_error;

/// This struct wraps the gpu buffer. can be used for array or element bindings
pub struct Buffer {
    pub id: glow::NativeBuffer,
    gl: Rc<glow::Context>,
}

impl Buffer {
    pub fn new(gl: Rc<glow::Context>) -> Buffer {
        unsafe {
            let id = gl.create_buffers().expect("failed to create buffer");

            Buffer { id, gl }
        }
    }
    pub fn update(&self, data: &[u8], usage: u32) {
        unsafe { self.gl.named_buffer_data_u8_slice(self.id, data, usage) }
    }
    pub fn bind(&self, target: u32) {
        unsafe {
            self.gl.bind_buffer(target, Some(self.id));
        }
    }
    pub fn unbind(&self, target: u32) {
        unsafe {
            self.gl.bind_buffer(target, None);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.id);
        }
    }
}
/// The vertex array attribute expressed as the attribute type, count and normalized.
#[derive(Debug, Default, Clone, Copy)]
pub struct VertexBufferLayoutElement {
    pub etype: u32,
    pub count: i32,
    pub normalized: bool,
}
/// This is used to set the vertex array attribute format/layout by implementing the Layout trait on any struct we would like to send to a gpu buffer
#[derive(Debug, Default, Clone)]
pub struct VertexBufferLayout {
    layout_of_elements: Vec<VertexBufferLayoutElement>,
}
/// we will avoid u16 types for now to keep alignment simple. u8 is for rgba egui until they use bytemuck
impl VertexBufferLayout {
    /// add a vao attribute that contains floats of count number. count must be equal to or less than 4. until we think of using vertices as matrices
    pub fn push_f32(&mut self, count: i32, normalized: bool) {
        self.layout_of_elements.push(VertexBufferLayoutElement {
            etype: glow::FLOAT,
            count,
            normalized,
        });
    }
    // pub fn push_u16(&mut self, count: i32) {
    //     &self.layout_of_elements.push(VertexBufferLayoutElement {
    //         etype: glow::UNSIGNED_SHORT,
    //         count,
    //         normalized: false,
    //     });
    // }
    /// The attribute made up of u8, will be expressed as a float in the shader. the gpu will auto convert it.  count must not be more than 4.
    pub fn push_u8(&mut self, count: i32, normalized: bool) {
        self.layout_of_elements.push(VertexBufferLayoutElement {
            etype: glow::UNSIGNED_BYTE,
            count,
            normalized,
        });
    }
    /// adds a vao attribute of type u32 with count number. count must not be more than 4.
    pub fn push_u32(&mut self, count: i32, normalized: bool) {
        self.layout_of_elements.push(VertexBufferLayoutElement {
            etype: glow::UNSIGNED_INT,
            count,
            normalized,
        });
    }

    /// this will take in a vao, and make sure to set buffer layout based on what we pushed on to it previously. uses dsa, so should not bind.
    pub fn set_layout(&self, gl: Rc<glow::Context>, vao: NativeVertexArray) {
        let mut stride: i32 = 0;
        for element in self.layout_of_elements.iter() {
            match element.etype {
                FLOAT | UNSIGNED_INT => {
                    stride += 4 * element.count as i32;
                }
                UNSIGNED_BYTE => {
                    stride += 1 * element.count as i32;
                }
                UNSIGNED_SHORT => {
                    stride += 2 * element.count as i32;
                }
                rest @ _ => {
                    panic!("vertexBufferElement's etype is not right: {}", rest);
                }
            }
        }
        let _stride = stride;
        let mut offset = 0;

        for (index, element) in self.layout_of_elements.iter().enumerate() {
            let index = index as u32;

            unsafe {
                //enabled the vertex array attribute
                gl.enable_vertex_attrib_array(index as u32);
                gl_error!(gl);

                // set the source for that vertex attribute data from the buffer bound at binding index 0 of the vao
                gl.vertex_array_attrib_binding_f32(vao, index, 0);
                gl_error!(gl);

                // set the attribute format according to the element type
                match element.etype {
                    FLOAT => {
                        gl.vertex_array_attrib_format_f32(
                            vao,
                            index,
                            element.count,
                            FLOAT,
                            element.normalized,
                            offset,
                        );
                        gl_error!(gl);

                        offset += 4 * element.count as u32;
                    }
                    UNSIGNED_INT => {
                        gl.vertex_array_attrib_format_i32(
                            vao,
                            index,
                            element.count,
                            UNSIGNED_INT,
                            offset,
                        );
                        gl_error!(gl);

                        offset += 4 * element.count as u32;
                    }
                    UNSIGNED_BYTE => {
                        gl.vertex_array_attrib_format_f32(
                            vao,
                            index,
                            element.count,
                            UNSIGNED_BYTE,
                            element.normalized,
                            offset,
                        );
                        gl_error!(gl);

                        offset += 1 * element.count as u32;
                    }
                    // UNSIGNED_SHORT => {
                    //     gl.vertex_array_attrib_format_f32(
                    //         index.try_into().unwrap(),
                    //         element.count,
                    //         UNSIGNED_SHORT,
                    //         stride,
                    //         offset,
                    //     );
                    //     offset += 2 * element.count as i32;
                    // }
                    _ => {
                        panic!("vertexBufferElement's etype is not right");
                    }
                }
            }
        }
    }
}

/// implement the trait for objects like vertices that you plan to send to gpu.
pub trait VertexBufferLayoutTrait {
    fn get_layout() -> VertexBufferLayout;
}
