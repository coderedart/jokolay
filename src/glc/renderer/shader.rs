use glow::*;
use std::{fs::File, io::Read, path::Path};

pub struct ShaderProgram<'a> {
    pub id: u32,
    gl: &'a glow::Context,
}

impl ShaderProgram<'_> {
    pub fn new<'a>(
        gl: &'a glow::Context,
        vertex_shader_src_path: &Path,
        fragment_shader_src_path: &Path,
    ) -> ShaderProgram<'a> {
        let mut vertex_shader_source: String = String::new();
        let mut fragment_shader_source = String::new();
        print!("{:?}", std::env::current_dir().unwrap());
        {
            File::open(vertex_shader_src_path)
                .expect("couldn't find shader.vs ")
                .read_to_string(&mut vertex_shader_source)
                .unwrap();
            File::open(fragment_shader_src_path)
                .expect("couldn't find shader.fs")
                .read_to_string(&mut fragment_shader_source)
                .unwrap();
        }
        unsafe {
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            let frag_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(vertex_shader, &vertex_shader_source);
            gl.compile_shader(vertex_shader);
            gl.shader_source(frag_shader, &fragment_shader_source);
            gl.compile_shader(frag_shader);
            let shader_program = gl.create_program().unwrap();
            gl.attach_shader(shader_program, vertex_shader);
            gl.attach_shader(shader_program, frag_shader);
            gl.link_program(shader_program);
            gl.delete_shader(vertex_shader);
            gl.delete_shader(frag_shader);
            ShaderProgram {
                id: shader_program,
                gl,
            }
        }
    }

    pub fn bind(&self) {
        unsafe { self.gl.use_program(Some(self.id)) }
    }
}

impl<'a> Drop for ShaderProgram<'a> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.id);
        }
    }
}
