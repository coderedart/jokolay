pub struct Painter {
    pub vao: VertexArrayObject,
    pub sp: ShaderProgram,
    pub vb: Buffer,
    pub ib: Buffer,
    pub u_mvp: UniformLocation,
    pub font_texture: Texture,
    pub gl: Rc<glow::Context>,
}

impl Painter {
    pub fn new(gl: Rc<glow::Context>, ctx: &mut imgui::Context) -> Self {
        let vao = VertexArrayObject::new(gl.clone());
        let vb = Buffer::new(gl.clone(), glow::ARRAY_BUFFER);
        let ib = Buffer::new(gl.clone(), glow::ELEMENT_ARRAY_BUFFER);
        let program = ShaderProgram::new(gl.clone(), VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None);
        let texture: Texture;
        {
            let mut atlas = ctx.fonts();
            // atlas.add_font(&[
            //     FontSource::DefaultFontData {
            //         config: Some(FontConfig {
            //             size_pixels: 13.0,
            //             ..FontConfig::default()
            //         }),
            //     },
            //     FontSource::TtfData {
            //         data: include_bytes!("../../../res/sans.ttf"),
            //         size_pixels: 13.0,
            //         config: Some(FontConfig {
            //             rasterizer_multiply: 0.75,
            //             glyph_ranges: FontGlyphRanges::default(),
            //             ..FontConfig::default()
            //         }),
            //     },
            // ]);
            let pixels = atlas.build_rgba32_texture();

            texture = Texture::new(gl.clone());
            texture.bind();
            unsafe {
                gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, 0);
            }
            texture.update_pixels(pixels.data, pixels.width, pixels.height);
            atlas.tex_id = (texture.id as usize).into();
        }
        program.bind();
        let u_mvp = unsafe { gl.get_uniform_location(program.id, "mvp") }.unwrap();

        let imgui_painter = Painter {
            vao,
            vb,
            ib,
            sp: program,
            gl: gl.clone(),
            u_mvp,
            font_texture: texture,
        };
        imgui_painter.bind();
        let layout = DrawVert::get_layout();
        layout.set_layout(gl.clone());

        return imgui_painter;
    }

    pub fn draw_meshes(&mut self, ui: imgui::Ui) {
        self.bind();

        unsafe {
            self.gl.enable(glow::BLEND);
            self.gl.blend_equation(glow::FUNC_ADD);
            self.gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            self.gl.disable(glow::DEPTH_TEST);
            self.gl.disable(glow::CULL_FACE);
            self.gl.enable(glow::SCISSOR_TEST);
            self.gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
        }
        let [width, height] = ui.io().display_size;

        let draw_data = ui.render();
        // let mvp: Mat4 = nalgebra_glm::ortho(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
        let mvp = [
            2.0 / width as f32, 0.0, 0.0, 0.0,
            0.0, 2.0 / -(height as f32), 0.0, 0.0,
            0.0, 0.0, -1.0, 0.0,
            -1.0, 1.0, 0.0, 1.0,
        ];
        unsafe {
            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.u_mvp),
                false,
                &mvp,
            );
        }
        for draw_list in draw_data.draw_lists() {
            self.draw_mesh(draw_list, width, height);
        }
    }
    pub fn draw_mesh(&self, draw_list: &DrawList, _width: f32, height: f32) {
        let vertices: Vec<VertexRgba> = draw_list
            .vtx_buffer()
            .iter()
            .map(|v| VertexRgba::from(v))
            .collect();
        let indices = draw_list.idx_buffer();

        self.update_buffers(
            Some((bytemuck::cast_slice(&vertices), glow::DYNAMIC_DRAW)),
            Some((bytemuck::cast_slice(indices), glow::DYNAMIC_DRAW)),
        );

        for cmd in draw_list.commands() {
            match cmd {
                DrawCmd::Elements {
                    count,
                    cmd_params:
                        DrawCmdParams {
                            clip_rect: [x, y, z, w],
                            texture_id,
                            idx_offset,
                            ..
                        },
                } => {
                    if texture_id.id() as u32 != self.font_texture.id {
                        panic!("wrong tex");
                    }
                    unsafe {
                        self.gl.scissor(
                            x as i32,
                            (height - w) as i32,
                            (z - x) as i32,
                            (w - y) as i32,
                        );

                        self.gl.draw_elements(
                            glow::TRIANGLES,
                            count as i32,
                            glow::UNSIGNED_SHORT,
                            (idx_offset * std::mem::size_of::<DrawIdx>()) as i32,
                        );
                    }
                }
                DrawCmd::ResetRenderState => {
                    unimplemented!("Haven't implemented DrawCmd::ResetRenderState yet");
                }
                DrawCmd::RawCallback { .. } => {
                    unimplemented!("Haven't implemented user callbacks yet");
                }
            }
        }
    }

    fn bind(&self) {
        self.vao.bind();
        self.vb.bind();
        self.ib.bind();
        self.sp.bind();
        self.font_texture.bind();
    }

    fn update_buffers(&self, vb: Option<(&[u8], u32)>, ib: Option<(&[u8], u32)>) {
        if let Some((data, usage)) = vb {
            self.vb.update(data, usage);
        }
        if let Some((data, usage)) = ib {
            self.ib.update(data, usage);
        }
    }

    fn _unbind(&self) {
        self.vao.unbind();
        self.vb.unbind();
        self.ib.unbind();
        self.sp.unbind();
    }
}

use std::rc::Rc;

use egui::{epaint::Vertex, Pos2};
use glow::{HasContext, UniformLocation};
use imgui::{DrawCmd, DrawCmdParams, DrawIdx, DrawList, DrawVert};


use crate::gltypes::{
    buffer::{Buffer, VertexBufferLayout, VertexBufferLayoutTrait},
    shader::ShaderProgram,
    texture::Texture,
    vertex_array::VertexArrayObject,
};

#[derive(Debug, Clone, Copy)]
pub struct VertexRgba {
    pub pos: Pos2,
    pub uv: Pos2,
    pub color: [u8; 4],
}

impl VertexBufferLayoutTrait for VertexRgba {
    fn get_layout() -> VertexBufferLayout {
        let mut vbl = VertexBufferLayout::default();
        vbl.push_f32(2, false);
        vbl.push_f32(2, false);
        vbl.push_u8(4, true);
        vbl
    }
}
impl From<&Vertex> for VertexRgba {
    fn from(vert: &Vertex) -> Self {
        VertexRgba {
            pos: vert.pos,
            uv: vert.uv,
            color: vert.color.to_array(),
        }
    }
}
impl From<Vertex> for VertexRgba {
    fn from(vert: Vertex) -> Self {
        VertexRgba {
            pos: vert.pos,
            uv: vert.uv,
            color: vert.color.to_array(),
        }
    }
}
impl From<&DrawVert> for VertexRgba {
    fn from(vert: &DrawVert) -> Self {
        VertexRgba {
            pos: vert.pos.into(),
            uv: vert.uv.into(),
            color: vert.col,
        }
    }
}
unsafe impl bytemuck::Zeroable for VertexRgba {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for VertexRgba {}

const FRAGMENT_SHADER_SRC: &str = r#"
#version 450

uniform sampler2D u_sampler;

in vec2 v_tc;
in vec4 v_color;
out vec4 f_color;

void main() {
  f_color =  v_color * texture(u_sampler, v_tc) ;
}"#;

const VERTEX_SHADER_SRC: &str = r#"
#version 450

uniform mat4 mvp;

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 tc;
layout(location = 2) in vec4 color;

out vec2 v_tc;
out vec4 v_color;

void main() {

    gl_Position = mvp * vec4(pos.xy, 0.0, 1.0);
    v_tc = tc;
    v_color = color;

}
"#;

impl VertexBufferLayoutTrait for DrawVert {
    fn get_layout() -> VertexBufferLayout {
        let mut layout = VertexBufferLayout::default();
        layout.push_f32(2, false);
        layout.push_f32(2, false);
        layout.push_u8(4, true);
        layout
    }
}
