use crate::core::graphic::camera::CameraBase;
use crate::core::graphic::hal::backend::{
    RendererApi, Texture, UniformBuffer, ShaderProgram,
};
use crate::core::graphic::hal::uniform_buffer::UniformBufferCommon;
use crate::core::graphic::shader::attribute::Attribute;
use crate::core::graphic::shader::shader_source::ShaderSource;
use gfx_hal::pso::Descriptor;
use nalgebra_glm::Mat4;
use gfx_hal::Backend;

pub struct Standard2DShaderProgram {
    shader_program: ShaderProgram,
    mvp_matrix_uniform: UniformBuffer<Mat4>,
}

impl Standard2DShaderProgram {
    pub fn new(api: &RendererApi, camera: &CameraBase) -> Self {
        let shader_source = ShaderSource::new(
            include_bytes!("../../../../target/data/shaders/standard.vert"),
            include_bytes!("../../../../target/data/shaders/standard.frag"),
        )
        .unwrap();

        let mvp_matrix: Mat4 = camera.borrow_combine().clone_owned();
        let attributes = create_2d_attributes();
        let descriptor_sets = create_2d_descriptor_set_layout_bindings();
        let shader = api.create_shader(shader_source, attributes, descriptor_sets);
        let shader_program = api.create_shader_program(shader);
        let mvp_matrix_uniform = api.create_uniform_buffer(&[mvp_matrix]);
        Standard2DShaderProgram {
            shader_program,
            mvp_matrix_uniform,
        }
    }

    pub fn borrow_shader_program(&self) -> &ShaderProgram {
        &self.shader_program
    }

    pub fn prepare(&mut self, mvp_matrix: &Mat4, texture: &Texture) {
        self.mvp_matrix_uniform
            .copy_to_buffer(&[mvp_matrix.clone_owned()]);
    }

    pub fn borrow_mvp_matrix_uniform(&self) -> &UniformBuffer<Mat4> {
        &self.mvp_matrix_uniform
    }
    /*pub fn write_descriptor_sets<'a>(
        &self,
        descriptor_set: &FixedBackend::DescriptorSet,
    ) -> Vec<gfx_hal::pso::DescriptorSetWrite<B, Option<Descriptor<B>>>> {
        vec![
            gfx_hal::pso::DescriptorSetWrite {
                set: descriptor_set,
                binding: 0,
                array_offset: 0,
                descriptors: Some(gfx_hal::pso::Descriptor::Image(
                    &*image_srv,
                    i::Layout::ShaderReadOnlyOptimal,
                )),
            },
            gfx_hal::pso::DescriptorSetWrite {
                set: &descriptor_set,
                binding: 1,
                array_offset: 0,
                descriptors: Some(gfx_hal::pso::Descriptor::Sampler(&*sampler)),
            },
        ]
    }*/
}

pub fn write_descriptor_sets<'a, B: Backend>(
    descriptor_set: &'a B::DescriptorSet,
    uniform: &'a UniformBufferCommon<B, Mat4>,
    image_view: &'a B::ImageView,
    sampler: &'a B::Sampler,
) -> Vec<gfx_hal::pso::DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>>> {
    vec![
        gfx_hal::pso::DescriptorSetWrite {
            set: &descriptor_set,
            binding: 0,
            array_offset: 0,
            descriptors: Some(gfx_hal::pso::Descriptor::Buffer(
                uniform.borrow_buffer(),
                gfx_hal::buffer::SubRange::WHOLE,
            )),
        },
        gfx_hal::pso::DescriptorSetWrite {
            set: descriptor_set,
            binding: 1,
            array_offset: 0,
            descriptors: Some(gfx_hal::pso::Descriptor::CombinedImageSampler(
                &*image_view,
                gfx_hal::image::Layout::ShaderReadOnlyOptimal,
                sampler,
            )),
        },
    ]
}

fn create_2d_attributes() -> Vec<Attribute> {
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
    ]
}

fn create_2d_descriptor_set_layout_bindings() -> Vec<gfx_hal::pso::DescriptorSetLayoutBinding> {
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
            stage_flags: gfx_hal::pso::ShaderStageFlags::FRAGMENT
                | gfx_hal::pso::ShaderStageFlags::VERTEX,
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
    ]
}
