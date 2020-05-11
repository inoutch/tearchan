use crate::core::graphic::hal::shader::Shader;
use crate::core::graphic::shader::attribute::Attribute;
use gfx_hal::device::Device;
use gfx_hal::pso::{CreationError, DescriptorPool, DescriptorType, DepthTest, Comparison};
use gfx_hal::Backend;
use std::borrow::Borrow;
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

pub struct GraphicPipeline<B: Backend> {
    descriptor_set_uniform_providers: Vec<i32>,
    descriptor_set_texture_providers: Vec<i32>,

    // native
    device: Weak<B::Device>,
    descriptor_pool: ManuallyDrop<B::DescriptorPool>,
    descriptor_set: B::DescriptorSet,
    descriptor_set_layout: ManuallyDrop<B::DescriptorSetLayout>,
    pipeline_layout: ManuallyDrop<B::PipelineLayout>,
    pipeline: ManuallyDrop<B::GraphicsPipeline>,
}

impl<B: Backend> GraphicPipeline<B> {
    pub fn new(device: &Rc<B::Device>, render_pass: &B::RenderPass, shader: &Shader<B>) -> Self {
        let descriptor_ranges = create_default_descriptor_range_descriptors();
        let mut descriptor_pool = unsafe {
            device
                .create_descriptor_pool(
                    64,
                    descriptor_ranges,
                    gfx_hal::pso::DescriptorPoolCreateFlags::empty(),
                )
                .unwrap()
        };

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(shader.borrow_descriptor_set_layout_bindings(), &[])
                .unwrap()
        };
        let descriptor_set = unsafe {
            descriptor_pool
                .allocate_set(&descriptor_set_layout)
                .unwrap()
        };

        let mut descriptor_set_layouts = vec![descriptor_set_layout];
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&descriptor_set_layouts, &[])
                .unwrap()
        };
        let subpass = gfx_hal::pass::Subpass {
            index: 0,
            main_pass: render_pass,
        };
        let mut pipeline_desc = gfx_hal::pso::GraphicsPipelineDesc::new(
            shader.create_entries(),
            gfx_hal::pso::Primitive::TriangleList,
            gfx_hal::pso::Rasterizer::FILL,
            &pipeline_layout,
            subpass,
        );
        pipeline_desc.vertex_buffers =
            convert_to_input_attribute_descriptions(shader.borrow_attributes());

        pipeline_desc
            .blender
            .targets
            .push(gfx_hal::pso::ColorBlendDesc {
                mask: gfx_hal::pso::ColorMask::ALL,
                blend: Some(gfx_hal::pso::BlendState::ALPHA),
            });

        pipeline_desc.depth_stencil.depth = Some(DepthTest {
            fun: Comparison::LessEqual,
            write: true,
        });
        pipeline_desc.depth_stencil.depth_bounds = true;

        shader.borrow_attributes().iter().for_each(|x| {
            pipeline_desc.attributes.push(x.attribute_desc);
        });

        let pipeline = unsafe {
            device
                .create_graphics_pipeline(&pipeline_desc, None)
                .unwrap()
        };
        GraphicPipeline {
            descriptor_set_uniform_providers: vec![],
            descriptor_set_texture_providers: vec![],
            device: Rc::downgrade(device),
            descriptor_pool: ManuallyDrop::new(descriptor_pool),
            descriptor_set,
            descriptor_set_layout: ManuallyDrop::new(descriptor_set_layouts.remove(0)),
            pipeline_layout: ManuallyDrop::new(pipeline_layout),
            pipeline: ManuallyDrop::new(pipeline),
        }
    }

    pub fn borrow_pipeline(&self) -> &B::GraphicsPipeline {
        self.pipeline.borrow()
    }

    pub fn borrow_pipeline_layout(&self) -> &B::PipelineLayout {
        self.pipeline_layout.borrow()
    }

    pub fn borrow_descriptor_set(&self) -> &B::DescriptorSet {
        &self.descriptor_set
    }
}

impl<B: Backend> Drop for GraphicPipeline<B> {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                device.destroy_graphics_pipeline(ManuallyDrop::into_inner(std::ptr::read(
                    &self.pipeline,
                )));
                device.destroy_pipeline_layout(ManuallyDrop::into_inner(std::ptr::read(
                    &self.pipeline_layout,
                )));
                device.destroy_descriptor_pool(ManuallyDrop::into_inner(std::ptr::read(
                    &self.descriptor_pool,
                )));
                device.destroy_descriptor_set_layout(ManuallyDrop::into_inner(std::ptr::read(
                    &self.descriptor_set_layout,
                )));
            }
        }
    }
}

fn create_default_descriptor_range_descriptors() -> Vec<gfx_hal::pso::DescriptorRangeDesc> {
    vec![
        gfx_hal::pso::DescriptorRangeDesc {
            ty: gfx_hal::pso::DescriptorType::Buffer {
                ty: gfx_hal::pso::BufferDescriptorType::Uniform,
                format: gfx_hal::pso::BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            },
            count: 32,
        },
        gfx_hal::pso::DescriptorRangeDesc {
            ty: gfx_hal::pso::DescriptorType::Sampler,
            count: 32,
        },
        gfx_hal::pso::DescriptorRangeDesc {
            ty: gfx_hal::pso::DescriptorType::Image {
                ty: gfx_hal::pso::ImageDescriptorType::Sampled { with_sampler: true },
            },
            count: 32,
        },
    ]
}

fn find_shader_stage_flag(descriptor_type: DescriptorType) -> gfx_hal::pso::ShaderStageFlags {
    match descriptor_type {
        DescriptorType::Sampler => gfx_hal::pso::ShaderStageFlags::FRAGMENT,
        DescriptorType::Image { .. } => gfx_hal::pso::ShaderStageFlags::FRAGMENT,
        DescriptorType::Buffer { .. } => gfx_hal::pso::ShaderStageFlags::GRAPHICS,
        DescriptorType::InputAttachment => gfx_hal::pso::ShaderStageFlags::GRAPHICS,
    }
}

fn convert_to_descriptors_set_layout_binding(
    descriptor_range: &gfx_hal::pso::DescriptorRangeDesc,
) -> gfx_hal::pso::DescriptorSetLayoutBinding {
    gfx_hal::pso::DescriptorSetLayoutBinding {
        binding: 0,
        ty: descriptor_range.ty,
        count: 1,
        stage_flags: find_shader_stage_flag(descriptor_range.ty),
        immutable_samplers: false,
    }
}

fn convert_to_input_attribute_descriptions(
    attributes: &[Attribute],
) -> Vec<gfx_hal::pso::VertexBufferDesc> {
    attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| gfx_hal::pso::VertexBufferDesc {
            binding: i as u32,
            stride: attr.stride,
            rate: gfx_hal::pso::VertexInputRate::Vertex,
        })
        .collect()
}
