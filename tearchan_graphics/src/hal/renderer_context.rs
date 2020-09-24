use crate::hal::buffer::index_buffer::IndexBufferCommon;
use crate::hal::buffer::vertex_buffer::VertexBufferCommon;
use crate::hal::frame_resource::FrameResource;
use crate::hal::graphic_pipeline::GraphicPipelineCommon;
use crate::hal::render_bundle::RenderBundleCommon;
use crate::hal::render_pass::RenderPass;
use gfx_hal::buffer::{IndexBufferView, SubRange};
use gfx_hal::command::{ClearColor, ClearDepthStencil, ClearValue, CommandBuffer, SubpassContents};
use gfx_hal::IndexType;

pub struct RendererContextCommon<'a, B: gfx_hal::Backend> {
    render_bundle: &'a mut RenderBundleCommon<B>,
    primary_render_pass: &'a mut RenderPass<B>,
    framebuffer: &'a mut B::Framebuffer,
    frame_resource: &'a mut FrameResource<B>,
}

impl<'a, B: gfx_hal::Backend> RendererContextCommon<'a, B> {
    pub fn new(
        render_bundle: &'a mut RenderBundleCommon<B>,
        primary_render_pass: &'a mut RenderPass<B>,
        framebuffer: &'a mut B::Framebuffer,
        frame_resource: &'a mut FrameResource<B>,
    ) -> RendererContextCommon<'a, B> {
        RendererContextCommon {
            render_bundle,
            primary_render_pass,
            framebuffer,
            frame_resource,
        }
    }

    pub fn render_bundle(&self) -> &RenderBundleCommon<B> {
        self.render_bundle
    }

    pub fn render_bundle_mut(&mut self) -> &mut RenderBundleCommon<B> {
        self.render_bundle
    }

    pub fn primary_render_pass(&self) -> &RenderPass<B> {
        self.primary_render_pass
    }

    pub fn draw_elements(
        &mut self,
        graphic_pipeline: &GraphicPipelineCommon<B>,
        index_size: usize,
        index_buffer: &IndexBufferCommon<B>,
        vertex_buffers: &[&VertexBufferCommon<B>],
    ) {
        unsafe {
            self.frame_resource
                .command_buffer_mut()
                .bind_graphics_pipeline(graphic_pipeline.pipeline());
            self.frame_resource
                .command_buffer_mut()
                .bind_graphics_descriptor_sets(
                    graphic_pipeline.pipeline_layout(),
                    0,
                    std::iter::once(graphic_pipeline.descriptor_set().get()),
                    &[],
                );

            let vertex_native_buffers = vertex_buffers
                .iter()
                .map(|buffer| (buffer.get(), SubRange::WHOLE))
                .collect::<Vec<_>>();
            self.frame_resource
                .command_buffer_mut()
                .bind_vertex_buffers(0, vertex_native_buffers);
            self.frame_resource
                .command_buffer_mut()
                .bind_index_buffer(IndexBufferView {
                    buffer: index_buffer.get(),
                    range: SubRange::WHOLE,
                    index_type: IndexType::U32,
                });
            self.frame_resource
                .command_buffer_mut()
                .set_depth_bounds(-1.0..1.0);

            // == Draw
            let render_area = self.render_bundle().display_size().viewport.rect;
            self.frame_resource.command_buffer_mut().begin_render_pass(
                self.primary_render_pass.get(),
                self.framebuffer,
                render_area,
                &[
                    ClearValue {
                        color: ClearColor {
                            float32: [0.3, 0.3, 0.3, 1.0],
                        },
                    },
                    ClearValue {
                        depth_stencil: ClearDepthStencil {
                            depth: 1.0f32,
                            stencil: 0,
                        },
                    },
                ],
                SubpassContents::Inline,
            );
            self.frame_resource
                .command_buffer_mut()
                .draw_indexed(0..index_size as u32, 0, 0..1);
            self.frame_resource.command_buffer_mut().end_render_pass();
        }
    }
}
