use glow::Context;
use super::{shader::ShaderProgram, texture::Texture};
use std::{collections::BTreeMap, rc::Rc};


pub struct Material{
    pub program: ShaderProgram,
    pub texture: Vec<Texture>,
    pub uniforms: BTreeMap<MaterialUniforms, u32>,
    pub gl: Rc<Context>,
}

impl Material {

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