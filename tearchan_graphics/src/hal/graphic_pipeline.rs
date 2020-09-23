use crate::hal::render_bundle::RenderBundleCommon;
use crate::hal::render_pass::RenderPass;
use crate::hal::shader::attribute::Attribute;
use crate::hal::shader::descriptor_set::DescriptorSetCommon;
use crate::hal::shader::ShaderCommon;
use gfx_hal::device::Device;
use gfx_hal::pso::{
    BlendState, BufferDescriptorFormat, BufferDescriptorType, ColorBlendDesc, ColorMask,
    Comparison, DepthTest, DescriptorPool, DescriptorRangeDesc, DescriptorType,
    GraphicsPipelineDesc, ImageDescriptorType, InputAssemblerDesc, Primitive,
    PrimitiveAssemblerDesc, Rasterizer, VertexBufferDesc, VertexInputRate,
};
use gfx_hal::Backend;
use std::mem::ManuallyDrop;

pub struct GraphicPipelineConfig {
    pub rasterizer: Rasterizer,
    pub primitive: Primitive,
}

impl Default for GraphicPipelineConfig {
    fn default() -> Self {
        GraphicPipelineConfig {
            rasterizer: Rasterizer::FILL,
            primitive: Primitive::TriangleList,
        }
    }
}

pub struct GraphicPipelineCommon<B: Backend> {
    render_bundle: RenderBundleCommon<B>,
    descriptor_pool: ManuallyDrop<B::DescriptorPool>,
    descriptor_set: DescriptorSetCommon<B>,
    descriptor_set_layout: ManuallyDrop<B::DescriptorSetLayout>,
    pipeline_layout: ManuallyDrop<B::PipelineLayout>,
    pipeline: ManuallyDrop<B::GraphicsPipeline>,
}

impl<B: Backend> GraphicPipelineCommon<B> {
    pub fn new(
        render_bundle: &RenderBundleCommon<B>,
        render_pass: &RenderPass<B>,
        shader: &ShaderCommon<B>,
        config: GraphicPipelineConfig,
    ) -> Self {
        let descriptor_ranges = create_default_descriptor_range_descriptors();
        let mut descriptor_pool = unsafe {
            render_bundle
                .device()
                .create_descriptor_pool(
                    64,
                    descriptor_ranges,
                    gfx_hal::pso::DescriptorPoolCreateFlags::empty(),
                )
                .unwrap()
        };

        let descriptor_set_layout = unsafe {
            render_bundle
                .device()
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
            render_bundle
                .device()
                .create_pipeline_layout(&descriptor_set_layouts, &[])
                .unwrap()
        };
        let subpass = gfx_hal::pass::Subpass {
            index: 0,
            main_pass: render_pass.get(),
        };
        let vertex_buffers = convert_to_input_attribute_descriptions(shader.borrow_attributes());
        let attributes = shader
            .borrow_attributes()
            .iter()
            .map(|x| x.attribute_desc)
            .collect::<Vec<_>>();

        let mut pipeline_desc = GraphicsPipelineDesc::new(
            PrimitiveAssemblerDesc::Vertex {
                buffers: &vertex_buffers,
                attributes: &attributes,
                input_assembler: InputAssemblerDesc {
                    primitive: config.primitive,
                    with_adjacency: false,
                    restart_index: None,
                },
                vertex: shader.vs_entry(),
                geometry: None,
                tessellation: None,
            },
            config.rasterizer,
            Some(shader.fs_entry()),
            &pipeline_layout,
            subpass,
        );

        pipeline_desc.blender.targets.push(ColorBlendDesc {
            mask: ColorMask::ALL,
            blend: Some(BlendState::ALPHA),
        });

        pipeline_desc.depth_stencil.depth = Some(DepthTest {
            fun: Comparison::LessEqual,
            write: true,
        });
        pipeline_desc.depth_stencil.depth_bounds = true;

        let pipeline = unsafe {
            render_bundle
                .device()
                .create_graphics_pipeline(&pipeline_desc, None)
                .unwrap()
        };
        GraphicPipelineCommon {
            render_bundle: render_bundle.clone(),
            descriptor_pool: ManuallyDrop::new(descriptor_pool),
            descriptor_set: DescriptorSetCommon::new(descriptor_set),
            descriptor_set_layout: ManuallyDrop::new(descriptor_set_layouts.remove(0)),
            pipeline_layout: ManuallyDrop::new(pipeline_layout),
            pipeline: ManuallyDrop::new(pipeline),
        }
    }

    pub fn pipeline(&self) -> &B::GraphicsPipeline {
        &self.pipeline
    }

    pub fn pipeline_layout(&self) -> &B::PipelineLayout {
        &self.pipeline_layout
    }

    pub fn descriptor_set(&self) -> &DescriptorSetCommon<B> {
        &self.descriptor_set
    }
}

impl<B: Backend> Drop for GraphicPipelineCommon<B> {
    fn drop(&mut self) {
        unsafe {
            self.render_bundle
                .device()
                .destroy_graphics_pipeline(ManuallyDrop::into_inner(std::ptr::read(
                    &self.pipeline,
                )));
            self.render_bundle
                .device()
                .destroy_pipeline_layout(ManuallyDrop::into_inner(std::ptr::read(
                    &self.pipeline_layout,
                )));
            self.render_bundle
                .device()
                .destroy_descriptor_pool(ManuallyDrop::into_inner(std::ptr::read(
                    &self.descriptor_pool,
                )));
            self.render_bundle
                .device()
                .destroy_descriptor_set_layout(ManuallyDrop::into_inner(std::ptr::read(
                    &self.descriptor_set_layout,
                )));
        }
    }
}

fn create_default_descriptor_range_descriptors() -> Vec<gfx_hal::pso::DescriptorRangeDesc> {
    vec![
        DescriptorRangeDesc {
            ty: DescriptorType::Buffer {
                ty: BufferDescriptorType::Uniform,
                format: BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            },
            count: 32,
        },
        DescriptorRangeDesc {
            ty: DescriptorType::Sampler,
            count: 32,
        },
        DescriptorRangeDesc {
            ty: DescriptorType::Image {
                ty: ImageDescriptorType::Sampled { with_sampler: true },
            },
            count: 32,
        },
    ]
}

fn convert_to_input_attribute_descriptions(attributes: &[Attribute]) -> Vec<VertexBufferDesc> {
    attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| VertexBufferDesc {
            binding: i as u32,
            stride: attr.stride,
            rate: VertexInputRate::Vertex,
        })
        .collect()
}
