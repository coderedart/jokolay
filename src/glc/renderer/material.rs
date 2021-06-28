use glow::Context;
use super::{shader::ShaderProgram, texture::Texture};
use std::collections::BTreeMap;


pub struct Material<'a> {
    pub program: ShaderProgram<'a>,
    pub texture: Vec<Texture<'a>>,
    pub uniforms: BTreeMap<MaterialUniforms, u32>,
    pub gl: &'a Context,
}

impl Material<'_> {
    pub fn bind(&self) {
        self.program.bind();
    }

    pub fn unbind(&self) {
        self.program.unbind();
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaterialUniforms {
    EguiScreenSize,
    EguiEtexSampler,
    MarkerVP,
    MarkerCamPos,
    MarkerPlayerPos,
    MarkerSampler0,
    MarkerSampler4,
    MarkerSampler8,
    MarkerSampler12,

}