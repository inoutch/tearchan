use crate::camera::Camera;
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
use nalgebra_glm::{vec3, Mat4, Vec3};

pub struct Standard3DShaderProgram {
    shader: Shader,
    vp_matrix_uniform: UniformBuffer<Mat4>,
    light_position_uniform: UniformBuffer<Vec3>,
    light_color_uniform: UniformBuffer<Vec3>,
    ambient_strength_uniform: UniformBuffer<f32>,
}

impl Standard3DShaderProgram {
    pub fn new(render_bundle: &RenderBundle, camera: &Camera) -> Self {
        let shader_source = ShaderSource::new(
            include_bytes!("../../../target/shaders/standard_3d.vert"),
            include_bytes!("../../../target/shaders/standard_3d.frag"),
        )
        .unwrap();

        let mvp_matrix: Mat4 = camera.combine().clone_owned();
        let attributes = create_3d_attributes();
        let descriptor_sets = create_3d_descriptor_set_layout_bindings();
        let shader = Shader::new(render_bundle, shader_source, attributes, descriptor_sets);
        let vp_matrix_uniform = UniformBuffer::new(render_bundle, &[mvp_matrix]);
        let light_position_uniform =
            UniformBuffer::new(render_bundle, &[vec3(0.0f32, 0.0f32, 0.0f32)]);
        let light_color_uniform =
            UniformBuffer::new(render_bundle, &[vec3(1.0f32, 1.0f32, 1.0f32)]);
        let ambient_strength_uniform = UniformBuffer::new(render_bundle, &[0.0f32]);

        Standard3DShaderProgram {
            shader,
            vp_matrix_uniform,
            light_position_uniform,
            light_color_uniform,
            ambient_strength_uniform,
        }
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn prepare(
        &mut self,
        vp_matrix: &Mat4,
        light_position: &Vec3,
        light_color: &Vec3,
        ambient_strength: f32,
    ) {
        self.vp_matrix_uniform
            .copy_to_buffer(&[vp_matrix.clone_owned()]);
        self.light_position_uniform
            .copy_to_buffer(&[light_position.clone_owned()]);
        self.light_color_uniform
            .copy_to_buffer(&[light_color.clone_owned()]);
        self.ambient_strength_uniform
            .copy_to_buffer(&[ambient_strength]);
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
                descriptors: Some(Descriptor::Buffer(
                    self.vp_matrix_uniform.buffer(),
                    SubRange::WHOLE,
                )),
            },
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 1,
                array_offset: 0,
                descriptors: Some(Descriptor::CombinedImageSampler(
                    texture.image_resource().image_view(),
                    Layout::ShaderReadOnlyOptimal,
                    texture.sampler(),
                )),
            },
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 2,
                array_offset: 0,
                descriptors: Some(Descriptor::Buffer(
                    self.light_position_uniform.buffer(),
                    SubRange::WHOLE,
                )),
            },
            DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 3,
                array_offset: 0,
                descriptors: Some(Descriptor::Buffer(
                    self.light_color_uniform.buffer(),
                    SubRange::WHOLE,
                )),
            },
            DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 4,
                array_offset: 0,
                descriptors: Some(Descriptor::Buffer(
                    self.ambient_strength_uniform.buffer(),
                    SubRange::WHOLE,
                )),
            },
        ])
    }
}

pub fn create_3d_attributes() -> Vec<Attribute> {
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
        Attribute {
            attribute_desc: AttributeDesc {
                // normal
                location: 3,
                binding: 3,
                element: Element {
                    format: Format::Rgb32Sfloat,
                    offset: 0,
                },
            },
            stride: 3 * std::mem::size_of::<f32>() as u32,
        },
    ]
}

fn create_3d_descriptor_set_layout_bindings() -> Vec<DescriptorSetLayoutBinding> {
    vec![
        DescriptorSetLayoutBinding {
            binding: 0,
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
            binding: 1,
            ty: DescriptorType::Image {
                ty: ImageDescriptorType::Sampled { with_sampler: true },
            },
            count: 1,
            stage_flags: ShaderStageFlags::FRAGMENT,
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
            stage_flags: ShaderStageFlags::GRAPHICS,
            immutable_samplers: false,
        },
        DescriptorSetLayoutBinding {
            binding: 3,
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
            binding: 4,
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
    ]
}
