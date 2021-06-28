use glow::*;
use std::{fs::File, io::Read, path::Path, rc::Rc};

/// Struct to abstract away creation/binding of shader program.
/// compiles shaders and attaches them to a new program. id is the program id.
/// destroys the program when dropped, so keep it alive if you don't want that.
pub struct ShaderProgram {
    pub id: u32,
    gl: Rc<glow::Context>,
}

impl ShaderProgram {
    /// takes in files containing vertex/fragment shaders and returns a Shaderprogram with them attached
    pub fn new(
        gl: Rc<glow::Context>,
        vertex_shader_src_path: &Path,
        geometry_shader_src_path: Option<&Path>,
        fragment_shader_src_path: &Path,
    ) -> ShaderProgram {
        let mut vertex_shader_source = String::new();
        let mut fragment_shader_source = String::new();
        let mut geometry_shader_source = String::new();
        {
            File::open(vertex_shader_src_path)
                .expect("couldn't find shader.vs ")
                .read_to_string(&mut vertex_shader_source)
                .unwrap();
            File::open(fragment_shader_src_path)
                .expect("couldn't find shader.fs")
                .read_to_string(&mut fragment_shader_source)
                .unwrap();
            if geometry_shader_src_path.is_some() {
                File::open(geometry_shader_src_path.unwrap())
                    .expect("couldn't find shader.fs")
                    .read_to_string(&mut geometry_shader_source)
                    .unwrap();
            }
        }

        unsafe {
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            let frag_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            let mut geometry_shader = 0;
            if geometry_shader_src_path.is_some() {
                geometry_shader = gl.create_shader(glow::GEOMETRY_SHADER).unwrap();
            }
            gl.shader_source(vertex_shader, &vertex_shader_source);
            gl.compile_shader(vertex_shader);
            if geometry_shader_src_path.is_some() {
                gl.shader_source(geometry_shader, &geometry_shader_source);
                gl.compile_shader(geometry_shader);
            }
            gl.shader_source(frag_shader, &fragment_shader_source);
            gl.compile_shader(frag_shader);
            let shader_program = gl.create_program().unwrap();
            gl.attach_shader(shader_program, vertex_shader);
            if geometry_shader_src_path.is_some() {
                gl.attach_shader(shader_program, geometry_shader);
            }
            gl.attach_shader(shader_program, frag_shader);
            gl.link_program(shader_program);
            gl.delete_shader(vertex_shader);
            if geometry_shader_src_path.is_some() {
                gl.delete_shader(geometry_shader);
            }
            gl.delete_shader(frag_shader);
            ShaderProgram {
                id: shader_program,
                gl,
            }
        }
    }
    /// makes the program the active one
    pub fn bind(&self) {
        unsafe { self.gl.use_program(Some(self.id)) }
    }
    pub fn unbind(&self) {
        unsafe {self.gl.use_program(None);}
    }

    pub fn get_uniform_id(&self, uniform_name: &str) -> Option<UniformLocation> {
        unsafe { self.gl.get_uniform_location(self.id, uniform_name) }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.id);
        }
    }
}
