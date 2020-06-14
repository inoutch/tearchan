use crate::core::graphic::camera::CameraBase;
use crate::core::graphic::hal::backend::{
    DescriptorSet, RendererApi, Shader, UniformBuffer, WriteDescriptorSets,
};
use crate::core::graphic::hal::shader::attribute::Attribute;
use crate::core::graphic::hal::shader::shader_source::ShaderSource;
use nalgebra_glm::Mat4;

pub struct GridShaderProgram {
    shader: Shader,
    mvp_matrix_uniform: UniformBuffer<Mat4>,
}

impl GridShaderProgram {
    pub fn new(api: &RendererApi, camera: &CameraBase) -> Self {
        let shader_source = ShaderSource::new(
            include_bytes!("../../../../target/data/shaders/grid.vert"),
            include_bytes!("../../../../target/data/shaders/grid.frag"),
        )
        .unwrap();

        let mvp_matrix: Mat4 = camera.combine().clone_owned();
        let attributes = create_grid_attributes();
        let descriptor_sets = create_grid_descriptor_set_layout_bindings();
        let shader = api.create_shader(shader_source, attributes, descriptor_sets);
        let mvp_matrix_uniform = api.create_uniform_buffer(&[mvp_matrix]);
        GridShaderProgram {
            shader,
            mvp_matrix_uniform,
        }
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn prepare(&mut self, mvp_matrix: &Mat4) {
        self.mvp_matrix_uniform
            .copy_to_buffer(&[mvp_matrix.clone_owned()]);
    }

    pub fn create_write_descriptor_sets<'a>(
        &'a self,
        descriptor_set: &'a DescriptorSet,
    ) -> WriteDescriptorSets<'a> {
        WriteDescriptorSets::new(vec![gfx_hal::pso::DescriptorSetWrite {
            set: descriptor_set.raw(),
            binding: 0,
            array_offset: 0,
            descriptors: Some(gfx_hal::pso::Descriptor::Buffer(
                self.mvp_matrix_uniform.buffer(),
                gfx_hal::buffer::SubRange::WHOLE,
            )),
        }])
    }
}

fn create_grid_attributes() -> Vec<Attribute> {
    vec![
        Attribute {
            attribute_desc: gfx_hal::pso::AttributeDesc {
                // position
                location: 0,
                binding: 0,
                element: gfx_hal::pso::Element {
                    format: gfx_hal::format::Format::Rgb32Sfloat,
                    offset: 0,
                },
            },
            stride: 3 * std::mem::size_of::<f32>() as u32,
        },
        Attribute {
            attribute_desc: gfx_hal::pso::AttributeDesc {
                // color
                location: 1,
                binding: 1,
                element: gfx_hal::pso::Element {
                    format: gfx_hal::format::Format::Rgba32Sfloat,
                    offset: 0,
                },
            },
            stride: 4 * std::mem::size_of::<f32>() as u32,
        },
    ]
}

fn create_grid_descriptor_set_layout_bindings() -> Vec<gfx_hal::pso::DescriptorSetLayoutBinding> {
    vec![gfx_hal::pso::DescriptorSetLayoutBinding {
        binding: 0,
        ty: gfx_hal::pso::DescriptorType::Buffer {
            ty: gfx_hal::pso::BufferDescriptorType::Uniform,
            format: gfx_hal::pso::BufferDescriptorFormat::Structured {
                dynamic_offset: false,
            },
        },
        count: 1,
        stage_flags: gfx_hal::pso::ShaderStageFlags::FRAGMENT
            | gfx_hal::pso::ShaderStageFlags::VERTEX,
        immutable_samplers: false,
    }]
}
