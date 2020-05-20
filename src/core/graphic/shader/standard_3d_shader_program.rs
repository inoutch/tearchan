use crate::core::graphic::camera::CameraBase;
use crate::core::graphic::hal::backend::{
    DescriptorSet, RendererApi, Shader, Texture, UniformBuffer, WriteDescriptorSets,
};
use crate::core::graphic::hal::shader::attribute::Attribute;
use crate::core::graphic::hal::shader::shader_source::ShaderSource;
use crate::math::mat::inverse_transpose;
use nalgebra_glm::{vec3, Mat4, Vec3};

pub struct Standard3DShaderProgram {
    shader: Shader,
    vp_matrix_uniform: UniformBuffer<Mat4>,
    inv_vp_matrix_uniform: UniformBuffer<Mat4>,
    light_position_uniform: UniformBuffer<Vec3>,
    light_color_uniform: UniformBuffer<Vec3>,
    ambient_strength_uniform: UniformBuffer<f32>,
}

impl Standard3DShaderProgram {
    pub fn new(api: &RendererApi, camera: &CameraBase) -> Self {
        let shader_source = ShaderSource::new(
            include_bytes!("../../../../target/data/shaders/standard_3d.vert"),
            include_bytes!("../../../../target/data/shaders/standard_3d.frag"),
        )
        .unwrap();
        let mvp_matrix: Mat4 = camera.combine().clone_owned();
        let attributes = create_3d_attributes();
        let descriptor_sets = create_3d_descriptor_set_layout_bindings();
        let shader = api.create_shader(shader_source, attributes, descriptor_sets);
        let vp_matrix_uniform = api.create_uniform_buffer(&[mvp_matrix]);
        let inv_vp_matrix_uniform =
            api.create_uniform_buffer(&[inverse_transpose(camera.combine().clone_owned())]);
        let light_position_uniform = api.create_uniform_buffer(&[vec3(0.0f32, 0.0f32, 0.0f32)]);
        let light_color_uniform = api.create_uniform_buffer(&[vec3(1.0f32, 1.0f32, 1.0f32)]);
        let ambient_strength_uniform = api.create_uniform_buffer(&[0.0f32]);
        Standard3DShaderProgram {
            shader,
            vp_matrix_uniform,
            inv_vp_matrix_uniform,
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
        _texture: &Texture,
    ) {
        self.vp_matrix_uniform
            .copy_to_buffer(&[vp_matrix.clone_owned()]);
        self.inv_vp_matrix_uniform
            .copy_to_buffer(&[inverse_transpose(vp_matrix.clone_owned())]);
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
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set.raw(),
                binding: 0,
                array_offset: 0,
                descriptors: Some(gfx_hal::pso::Descriptor::Buffer(
                    self.vp_matrix_uniform.buffer(),
                    gfx_hal::buffer::SubRange::WHOLE,
                )),
            },
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set.raw(),
                binding: 1,
                array_offset: 0,
                descriptors: Some(gfx_hal::pso::Descriptor::Buffer(
                    self.inv_vp_matrix_uniform.buffer(),
                    gfx_hal::buffer::SubRange::WHOLE,
                )),
            },
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set.raw(),
                binding: 2,
                array_offset: 0,
                descriptors: Some(gfx_hal::pso::Descriptor::CombinedImageSampler(
                    texture.image_view(),
                    gfx_hal::image::Layout::ShaderReadOnlyOptimal,
                    texture.sampler(),
                )),
            },
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set.raw(),
                binding: 3,
                array_offset: 0,
                descriptors: Some(gfx_hal::pso::Descriptor::Buffer(
                    self.light_position_uniform.buffer(),
                    gfx_hal::buffer::SubRange::WHOLE,
                )),
            },
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set.raw(),
                binding: 4,
                array_offset: 0,
                descriptors: Some(gfx_hal::pso::Descriptor::Buffer(
                    self.light_color_uniform.buffer(),
                    gfx_hal::buffer::SubRange::WHOLE,
                )),
            },
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set.raw(),
                binding: 5,
                array_offset: 0,
                descriptors: Some(gfx_hal::pso::Descriptor::Buffer(
                    self.ambient_strength_uniform.buffer(),
                    gfx_hal::buffer::SubRange::WHOLE,
                )),
            },
        ])
    }
}

fn create_3d_attributes() -> Vec<Attribute> {
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
        Attribute {
            attribute_desc: gfx_hal::pso::AttributeDesc {
                // texcoord
                location: 2,
                binding: 2,
                element: gfx_hal::pso::Element {
                    format: gfx_hal::format::Format::Rg32Sfloat,
                    offset: 0,
                },
            },
            stride: 2 * std::mem::size_of::<f32>() as u32,
        },
        Attribute {
            attribute_desc: gfx_hal::pso::AttributeDesc {
                // normal
                location: 3,
                binding: 3,
                element: gfx_hal::pso::Element {
                    format: gfx_hal::format::Format::Rgb32Sfloat,
                    offset: 0,
                },
            },
            stride: 3 * std::mem::size_of::<f32>() as u32,
        },
    ]
}

fn create_3d_descriptor_set_layout_bindings() -> Vec<gfx_hal::pso::DescriptorSetLayoutBinding> {
    vec![
        gfx_hal::pso::DescriptorSetLayoutBinding {
            binding: 0,
            ty: gfx_hal::pso::DescriptorType::Buffer {
                ty: gfx_hal::pso::BufferDescriptorType::Uniform,
                format: gfx_hal::pso::BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            },
            count: 1,
            stage_flags: gfx_hal::pso::ShaderStageFlags::GRAPHICS,
            immutable_samplers: false,
        },
        gfx_hal::pso::DescriptorSetLayoutBinding {
            binding: 1,
            ty: gfx_hal::pso::DescriptorType::Buffer {
                ty: gfx_hal::pso::BufferDescriptorType::Uniform,
                format: gfx_hal::pso::BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            },
            count: 1,
            stage_flags: gfx_hal::pso::ShaderStageFlags::GRAPHICS,
            immutable_samplers: false,
        },
        gfx_hal::pso::DescriptorSetLayoutBinding {
            binding: 2,
            ty: gfx_hal::pso::DescriptorType::Image {
                ty: gfx_hal::pso::ImageDescriptorType::Sampled { with_sampler: true },
            },
            count: 1,
            stage_flags: gfx_hal::pso::ShaderStageFlags::FRAGMENT,
            immutable_samplers: false,
        },
        gfx_hal::pso::DescriptorSetLayoutBinding {
            binding: 3,
            ty: gfx_hal::pso::DescriptorType::Buffer {
                ty: gfx_hal::pso::BufferDescriptorType::Uniform,
                format: gfx_hal::pso::BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            },
            count: 1,
            stage_flags: gfx_hal::pso::ShaderStageFlags::GRAPHICS,
            immutable_samplers: false,
        },
        gfx_hal::pso::DescriptorSetLayoutBinding {
            binding: 4,
            ty: gfx_hal::pso::DescriptorType::Buffer {
                ty: gfx_hal::pso::BufferDescriptorType::Uniform,
                format: gfx_hal::pso::BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            },
            count: 1,
            stage_flags: gfx_hal::pso::ShaderStageFlags::GRAPHICS,
            immutable_samplers: false,
        },
        gfx_hal::pso::DescriptorSetLayoutBinding {
            binding: 5,
            ty: gfx_hal::pso::DescriptorType::Buffer {
                ty: gfx_hal::pso::BufferDescriptorType::Uniform,
                format: gfx_hal::pso::BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            },
            count: 1,
            stage_flags: gfx_hal::pso::ShaderStageFlags::GRAPHICS,
            immutable_samplers: false,
        },
    ]
}
