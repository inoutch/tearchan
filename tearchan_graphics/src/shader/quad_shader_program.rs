use crate::hal::backend::{
    DescriptorSet, RenderBundle, Shader, Texture, UniformBuffer, WriteDescriptorSets,
};
use crate::hal::shader::attribute::Attribute;
use crate::hal::shader::shader_source::ShaderSource;
use gfx_hal::buffer::SubRange;
use gfx_hal::format::Format;
use gfx_hal::image::Layout;
use gfx_hal::pso::{
    AttributeDesc, BufferDescriptorFormat, BufferDescriptorType, Descriptor,
    DescriptorSetLayoutBinding, DescriptorSetWrite, DescriptorType, Element, ImageDescriptorType,
    ShaderStageFlags,
};
use nalgebra_glm::{vec4, Mat4, Vec4};

pub struct QuadShaderProgram {
    shader: Shader,
    color_uniform: UniformBuffer<Vec4>,
    vp_matrix_uniform: UniformBuffer<Mat4>,
}

impl QuadShaderProgram {
    pub fn new(render_bundle: &RenderBundle) -> Self {
        let shader_source = ShaderSource::new(
            include_bytes!("../../../target/shaders/quad.vert"),
            include_bytes!("../../../target/shaders/quad.frag"),
        )
        .unwrap();

        let color_uniform =
            UniformBuffer::new(render_bundle, &[vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32)]);
        let vp_matrix_uniform = UniformBuffer::new(render_bundle, &[Mat4::identity()]);
        let attributes = create_2d_attributes();
        let descriptor_sets = create_2d_descriptor_set_layout_bindings();
        let shader = Shader::new(render_bundle, shader_source, attributes, descriptor_sets);
        QuadShaderProgram {
            shader,
            color_uniform,
            vp_matrix_uniform,
        }
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn create_write_descriptor_sets<'a>(
        &'a self,
        descriptor_set: &'a DescriptorSet,
        texture: &'a Texture,
    ) -> WriteDescriptorSets<'a> {
        WriteDescriptorSets::new(vec![
            DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 0,
                array_offset: 0,
                descriptors: Some(Descriptor::CombinedImageSampler(
                    texture.image_resource().image_view(),
                    Layout::ShaderReadOnlyOptimal,
                    texture.sampler(),
                )),
            },
            DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 1,
                array_offset: 0,
                descriptors: Some(Descriptor::Buffer(
                    self.color_uniform.buffer(),
                    SubRange::WHOLE,
                )),
            },
            DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 2,
                array_offset: 0,
                descriptors: Some(Descriptor::Buffer(
                    self.vp_matrix_uniform.buffer(),
                    SubRange::WHOLE,
                )),
            },
        ])
    }
}

fn create_2d_attributes() -> Vec<Attribute> {
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
        Attribute {
            attribute_desc: AttributeDesc {
                // texcoord
                location: 2,
                binding: 2,
                element: Element {
                    format: Format::Rg32Sfloat,
                    offset: 0,
                },
            },
            stride: 2 * std::mem::size_of::<f32>() as u32,
        },
    ]
}

fn create_2d_descriptor_set_layout_bindings() -> Vec<DescriptorSetLayoutBinding> {
    vec![
        DescriptorSetLayoutBinding {
            binding: 0,
            ty: DescriptorType::Image {
                ty: ImageDescriptorType::Sampled { with_sampler: true },
            },
            count: 1,
            stage_flags: ShaderStageFlags::FRAGMENT,
            immutable_samplers: false,
        },
        DescriptorSetLayoutBinding {
            binding: 1,
            ty: DescriptorType::Buffer {
                ty: BufferDescriptorType::Uniform,
                format: BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            },
            count: 1,
            stage_flags: ShaderStageFlags::GRAPHICS,
            immutable_samplers: false,
        },
        DescriptorSetLayoutBinding {
            binding: 2,
            ty: DescriptorType::Buffer {
                ty: BufferDescriptorType::Uniform,
                format: BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            },
            count: 1,
            stage_flags: ShaderStageFlags::VERTEX,
            immutable_samplers: false,
        },
    ]
}
