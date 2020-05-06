use crate::core::graphic::hal::backend::FixedBackend;
use crate::core::graphic::hal::shader::Shader;
use gfx_hal::adapter::MemoryType;
use gfx_hal::Backend;
use std::rc::Rc;

pub struct ShaderProgramCommon<B: Backend> {
    shader: Shader<B>,
}

impl<B: Backend> ShaderProgramCommon<B> {
    pub fn new(
        device: &Rc<B::Device>,
        memory_types: &[MemoryType],
        shader: Shader<B>,
    ) -> ShaderProgramCommon<B> {
        let bindings = shader.borrow_descriptor_set_layout_bindings();
        ShaderProgramCommon { shader }
    }

    pub fn borrow_shader(&self) -> &Shader<B> {
        &self.shader
    }
}

pub type ShaderProgram = ShaderProgramCommon<FixedBackend>;
