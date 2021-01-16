use crate::{ShaderDesc, UniformBuffer, Device};
use nalgebra_glm::{Mat4, Vec4};

pub struct QuadShaderProgram {
    shader_desc: ShaderDesc,
    color_uniform: UniformBuffer<Vec4>,
    vp_matrix_uniform: UniformBuffer<Mat4>,
}

impl QuadShaderProgram {
    pub fn create(&self, device: Device) -> QuadShaderProgram {
        let shader_module = device.create_
    }
}
