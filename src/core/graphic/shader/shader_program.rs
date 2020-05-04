use crate::core::graphic::hal::backend::FixedBackend;
use crate::core::graphic::hal::shader::Shader;
use gfx_hal::Backend;

pub struct ShaderProgramCommon<B: Backend> {
    shader: Shader<B>,
    descriptor_sets: Vec<gfx_hal::pso::DescriptorSetLayoutBinding>,
}

impl<B: Backend> ShaderProgramCommon<B> {
    pub fn new(
        shader: Shader<B>,
        descriptor_sets: Vec<gfx_hal::pso::DescriptorSetLayoutBinding>,
    ) -> ShaderProgramCommon<B> {
        ShaderProgramCommon {
            shader,
            descriptor_sets,
        }
    }

    pub fn borrow_shader(&self) -> &Shader<B> {
        &self.shader
    }

    pub fn borrow_descriptor_sets(&self) -> &Vec<gfx_hal::pso::DescriptorSetLayoutBinding> {
        &self.descriptor_sets
    }
}

pub type ShaderProgram = ShaderProgramCommon<FixedBackend>;
