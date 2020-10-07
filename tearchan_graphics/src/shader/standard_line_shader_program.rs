use crate::hal::backend::{
    DescriptorSet, RenderBundle, Shader, UniformBuffer, WriteDescriptorSets,
};
use crate::hal::shader::attribute::Attribute;
use crate::hal::shader::shader_source::ShaderSource;
use gfx_hal::buffer::SubRange;
use gfx_hal::format::Format;
use gfx_hal::pso::{
    AttributeDesc, BufferDescriptorFormat, BufferDescriptorType, Descriptor,
    DescriptorSetLayoutBinding, DescriptorSetWrite, DescriptorType, Element, ShaderStageFlags,
};
use nalgebra_glm::Mat4;

pub struct StandardLineShaderProgram {
    shader: Shader,
    mvp_matrix_uniform: UniformBuffer<Mat4>,
}

impl StandardLineShaderProgram {
    pub fn new(render_bundle: &RenderBundle, mvp_matrix: Mat4) -> Self {
        let shader_source = ShaderSource::new(
            include_bytes!("../../../target/shaders/standard_line.vert"),
            include_bytes!("../../../target/shaders/standard_line.frag"),
        )
        .unwrap();

        let attributes = create_grid_attributes();
        let descriptor_sets = create_grid_descriptor_set_layout_bindings();
        let shader = Shader::new(render_bundle, shader_source, attributes, descriptor_sets);
        let mvp_matrix_uniform = UniformBuffer::new(render_bundle, &[mvp_matrix]);
        StandardLineShaderProgram {
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
        WriteDescriptorSets::new(vec![DescriptorSetWrite {
            set: descriptor_set.get(),
            binding: 0,
            array_offset: 0,
            descriptors: Some(Descriptor::Buffer(
                self.mvp_matrix_uniform.buffer(),
                SubRange::WHOLE,
            )),
        }])
    }
}

fn create_grid_attributes() -> Vec<Attribute> {
    vec![
        Attribute {
            attribute_desc: AttributeDesc {
                // position
                location: 0,
                binding: 0,
                element: Element {
                    format: Format::Rgb32Sfloat,
                    offset: 0,
                },
            },
            stride: 3 * std::mem::size_of::<f32>() as u32,
        },
        Attribute {
            attribute_desc: AttributeDesc {
                // color
                location: 1,
                binding: 1,
                element: Element {
                    format: Format::Rgba32Sfloat,
                    offset: 0,
                },
            },
            stride: 4 * std::mem::size_of::<f32>() as u32,
        },
    ]
}

fn create_grid_descriptor_set_layout_bindings() -> Vec<DescriptorSetLayoutBinding> {
    vec![DescriptorSetLayoutBinding {
        binding: 0,
        ty: DescriptorType::Buffer {
            ty: BufferDescriptorType::Uniform,
            format: BufferDescriptorFormat::Structured {
                dynamic_offset: false,
            },
        },
        count: 1,
        stage_flags: ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX,
        immutable_samplers: false,
    }]
}
