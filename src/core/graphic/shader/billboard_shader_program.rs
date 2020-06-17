use crate::core::graphic::camera::CameraBase;
use crate::core::graphic::hal::backend::{
    DescriptorSet, RendererApi, Shader, Texture, UniformBuffer, WriteDescriptorSets,
};
use crate::core::graphic::hal::shader::attribute::Attribute;
use crate::core::graphic::hal::shader::shader_source::ShaderSource;
use nalgebra_glm::{vec3, Mat4, Vec3};

struct BillboardCamera {
    pub camera_right: Vec3,
    _pad1: f32, // alignment
    pub camera_up: Vec3,
    _pad2: f32,
}

impl BillboardCamera {
    pub fn new(camera: &CameraBase) -> Self {
        BillboardCamera {
            camera_right: vec3(
                camera.view_matrix.data[0],
                camera.view_matrix.data[4],
                camera.view_matrix.data[8],
            ),
            camera_up: vec3(
                camera.view_matrix.data[1],
                camera.view_matrix.data[5],
                camera.view_matrix.data[9],
            ),
            _pad1: 0.0f32,
            _pad2: 0.0f32,
        }
    }
}

pub struct BillboardShaderProgram {
    shader: Shader,
    vp_matrix_uniform: UniformBuffer<Mat4>,
    billboard_camera_uniform: UniformBuffer<BillboardCamera>,
}

impl BillboardShaderProgram {
    pub fn new(api: &RendererApi, camera: &CameraBase) -> Self {
        let shader_source = ShaderSource::new(
            include_bytes!("../../../../target/data/shaders/billboard.vert"),
            include_bytes!("../../../../target/data/shaders/billboard.frag"),
        )
        .unwrap();

        let vp_matrix: Mat4 = camera.combine().clone_owned();
        let attributes = create_billboard_attributes();
        let descriptor_sets = create_billboard_descriptor_set_layout_bindings();
        let shader = api.create_shader(shader_source, attributes, descriptor_sets);

        let vp_matrix_uniform = api.create_uniform_buffer(&[vp_matrix]);
        let billboard_camera_uniform = api.create_uniform_buffer(&[BillboardCamera::new(camera)]);
        BillboardShaderProgram {
            shader,
            vp_matrix_uniform,
            billboard_camera_uniform,
        }
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn prepare(&mut self, camera: &CameraBase) {
        self.vp_matrix_uniform
            .copy_to_buffer(&[camera.combine().clone_owned()]);
        self.billboard_camera_uniform
            .copy_to_buffer(&[BillboardCamera::new(camera)]);
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
                descriptors: Some(gfx_hal::pso::Descriptor::CombinedImageSampler(
                    texture.image_view(),
                    gfx_hal::image::Layout::ShaderReadOnlyOptimal,
                    texture.sampler(),
                )),
            },
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set.raw(),
                binding: 2,
                array_offset: 0,
                descriptors: Some(gfx_hal::pso::Descriptor::Buffer(
                    self.billboard_camera_uniform.buffer(),
                    gfx_hal::buffer::SubRange::WHOLE,
                )),
            },
        ])
    }
}

fn create_billboard_attributes() -> Vec<Attribute> {
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

fn create_billboard_descriptor_set_layout_bindings() -> Vec<gfx_hal::pso::DescriptorSetLayoutBinding>
{
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
            ty: gfx_hal::pso::DescriptorType::Image {
                ty: gfx_hal::pso::ImageDescriptorType::Sampled { with_sampler: true },
            },
            count: 1,
            stage_flags: gfx_hal::pso::ShaderStageFlags::FRAGMENT,
            immutable_samplers: false,
        },
        gfx_hal::pso::DescriptorSetLayoutBinding {
            binding: 2,
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
