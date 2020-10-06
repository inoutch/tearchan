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

struct BillboardCamera {
    pub camera_right: Vec3,
    _pad1: f32, // alignment
    pub camera_up: Vec3,
    _pad2: f32,
}

impl BillboardCamera {
    pub fn new(camera: &Camera) -> Self {
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
    pub fn new(render_bundle: &RenderBundle, camera: &Camera) -> Self {
        BillboardShaderProgram::new_with_shader_source(
            render_bundle,
            camera,
            ShaderSource::new(
                include_bytes!("../../../target/shaders/billboard.vert"),
                include_bytes!("../../../target/shaders/billboard.frag"),
            )
            .unwrap(),
        )
    }

    pub fn new_with_alpha(render_bundle: &RenderBundle, camera: &Camera) -> Self {
        BillboardShaderProgram::new_with_shader_source(
            render_bundle,
            camera,
            ShaderSource::new(
                include_bytes!("../../../target/shaders/billboard.vert"),
                include_bytes!("../../../target/shaders/billboard_alpha.frag"),
            )
            .unwrap(),
        )
    }

    fn new_with_shader_source(
        render_bundle: &RenderBundle,
        camera: &Camera,
        shader_source: ShaderSource,
    ) -> Self {
        let vp_matrix: Mat4 = camera.combine().clone_owned();
        let attributes = create_billboard_attributes();
        let descriptor_sets = create_billboard_descriptor_set_layout_bindings();
        let shader = Shader::new(render_bundle, shader_source, attributes, descriptor_sets);

        let vp_matrix_uniform = UniformBuffer::new(render_bundle, &[vp_matrix]);
        let billboard_camera_uniform =
            UniformBuffer::new(render_bundle, &[BillboardCamera::new(camera)]);
        BillboardShaderProgram {
            shader,
            vp_matrix_uniform,
            billboard_camera_uniform,
        }
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn prepare(&mut self, camera: &Camera) {
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
            DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 0,
                array_offset: 0,
                descriptors: Some(Descriptor::Buffer(
                    self.vp_matrix_uniform.buffer(),
                    SubRange::WHOLE,
                )),
            },
            DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 1,
                array_offset: 0,
                descriptors: Some(Descriptor::CombinedImageSampler(
                    texture.image_resource().image_view(),
                    Layout::ShaderReadOnlyOptimal,
                    texture.sampler(),
                )),
            },
            DescriptorSetWrite {
                set: descriptor_set.get(),
                binding: 2,
                array_offset: 0,
                descriptors: Some(Descriptor::Buffer(
                    self.billboard_camera_uniform.buffer(),
                    SubRange::WHOLE,
                )),
            },
        ])
    }
}

fn create_billboard_attributes() -> Vec<Attribute> {
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

fn create_billboard_descriptor_set_layout_bindings() -> Vec<DescriptorSetLayoutBinding> {
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
    ]
}
