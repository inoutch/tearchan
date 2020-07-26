use crate::core::graphic::hal::graphic_pipeline::{GraphicPipelineCommon, GraphicPipelineConfig};
use crate::core::graphic::hal::index_buffer::IndexBufferCommon;
use crate::core::graphic::hal::renderer::DisplaySize;
use crate::core::graphic::hal::shader::attribute::Attribute;
use crate::core::graphic::hal::shader::shader_source::ShaderSource;
use crate::core::graphic::hal::shader::ShaderCommon;
use crate::core::graphic::hal::texture::{TextureCommon, TextureConfig};
use crate::core::graphic::hal::uniform_buffer::UniformBufferCommon;
use crate::core::graphic::hal::vertex_buffer::VertexBufferCommon;
use crate::core::graphic::hal::write_descriptor_sets::WriteDescriptorSetsCommon;
use crate::core::graphic::image::Image;
use crate::math::mesh::IndexType;
use gfx_hal::adapter::MemoryType;
use gfx_hal::buffer::{IndexBufferView, SubRange};
use gfx_hal::command::{ClearColor, ClearDepthStencil, ClearValue, CommandBuffer, SubpassContents};
use gfx_hal::device::Device;
use gfx_hal::queue::QueueGroup;
use gfx_hal::{Backend, Limits};
use nalgebra_glm::Vec4;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::rc::Rc;

pub struct GraphicsContext<B: Backend> {
    // gfx-hal instances
    pub device: Rc<B::Device>,
    pub queue_group: QueueGroup<B>,
    pub first_render_pass: ManuallyDrop<B::RenderPass>,
    pub render_pass: ManuallyDrop<B::RenderPass>,
    // properties
    pub use_first_render_pass: bool,
    pub clear_color: Vec4,
    pub clear_depth: f32,
    pub memory_types: Vec<MemoryType>,
    pub limits: Limits,
    pub display_size: DisplaySize,
}

pub struct GraphicsCommon<'a, B: Backend> {
    context: &'a mut GraphicsContext<B>,
    command_pool: &'a mut B::CommandPool,
    command_buffer: &'a mut B::CommandBuffer,
    frame_buffer: &'a B::Framebuffer,
}

impl<'a, B: gfx_hal::Backend> GraphicsCommon<'a, B> {
    pub fn new(
        context: &'a mut GraphicsContext<B>,
        command_pool: &'a mut B::CommandPool,
        command_buffer: &'a mut B::CommandBuffer,
        frame_buffer: &'a B::Framebuffer,
    ) -> GraphicsCommon<'a, B> {
        GraphicsCommon {
            context,
            command_pool,
            command_buffer,
            frame_buffer,
        }
    }

    pub fn create_vertex_buffer(&self, vertices: &[f32]) -> VertexBufferCommon<B> {
        VertexBufferCommon::new(&self.context.device, &self.context.memory_types, vertices)
    }

    pub fn create_index_buffer(&self, indices: &[IndexType]) -> IndexBufferCommon<B> {
        IndexBufferCommon::new(&self.context.device, &self.context.memory_types, indices)
    }

    pub fn create_uniform_buffer<T>(&self, data_source: &[T]) -> UniformBufferCommon<B, T> {
        UniformBufferCommon::new(
            &self.context.device,
            &self.context.memory_types,
            data_source,
        )
    }

    pub fn create_shader(
        &self,
        shader_source: ShaderSource,
        attributes: Vec<Attribute>,
        descriptor_sets: Vec<gfx_hal::pso::DescriptorSetLayoutBinding>,
    ) -> ShaderCommon<B> {
        debug_assert!(!attributes.is_empty(), "attributes are empty");
        ShaderCommon::new(
            &self.context.device,
            shader_source,
            attributes,
            descriptor_sets,
        )
    }

    pub fn create_texture(&mut self, image: &Image, config: TextureConfig) -> TextureCommon<B> {
        TextureCommon::new(
            &self.context.device,
            self.command_pool,
            &mut self.context.queue_group,
            &self.context.memory_types,
            &self.context.limits,
            image,
            config,
        )
    }

    pub fn create_graphic_pipeline(
        &mut self,
        shader: &ShaderCommon<B>,
        config: GraphicPipelineConfig,
    ) -> GraphicPipelineCommon<B> {
        GraphicPipelineCommon::new(
            &self.context.device,
            self.context.render_pass.deref(),
            shader,
            config,
        )
    }

    pub fn draw_vertices(
        &mut self,
        graphic_pipeline: &GraphicPipelineCommon<B>,
        vertex_buffers: &[&VertexBufferCommon<B>],
        vertices_size: usize,
    ) {
        unsafe {
            self.command_buffer
                .bind_graphics_pipeline(graphic_pipeline.pipeline());
            self.command_buffer.bind_graphics_descriptor_sets(
                graphic_pipeline.pipeline_layout(),
                0,
                std::iter::once(graphic_pipeline.descriptor_set().raw()),
                &[],
            );

            let buffers: Vec<(&B::Buffer, gfx_hal::buffer::SubRange)> = vertex_buffers
                .iter()
                .map(|x| (x.borrow_buffer(), gfx_hal::buffer::SubRange::WHOLE))
                .collect();
            self.command_buffer.bind_vertex_buffers(0, buffers);
            if self.context.use_first_render_pass {
                self.command_buffer.begin_render_pass(
                    self.context.first_render_pass.deref(),
                    self.frame_buffer,
                    self.context.display_size.viewport.rect,
                    &[
                        gfx_hal::command::ClearValue {
                            color: gfx_hal::command::ClearColor {
                                float32: [0.3, 0.3, 0.3, 1.0],
                            },
                        },
                        gfx_hal::command::ClearValue {
                            depth_stencil: ClearDepthStencil {
                                depth: 1.0f32,
                                stencil: 0,
                            },
                        },
                    ],
                    gfx_hal::command::SubpassContents::Inline,
                );
                self.context.use_first_render_pass = false;
            } else {
                self.command_buffer.begin_render_pass(
                    self.context.render_pass.deref(),
                    self.frame_buffer,
                    self.context.display_size.viewport.rect,
                    &[],
                    gfx_hal::command::SubpassContents::Inline,
                );
            }
            self.command_buffer.draw(0..vertices_size as u32, 0..1);
            self.command_buffer.end_render_pass();
        }
    }

    pub fn draw_elements(
        &mut self,
        graphic_pipeline: &GraphicPipelineCommon<B>,
        index_size: usize,
        index_buffer: &IndexBufferCommon<B>,
        vertex_buffers: &[&VertexBufferCommon<B>],
    ) {
        unsafe {
            // === Setup graphic pipeline
            self.command_buffer
                .bind_graphics_pipeline(graphic_pipeline.pipeline());
            self.command_buffer.bind_graphics_descriptor_sets(
                graphic_pipeline.pipeline_layout(),
                0,
                std::iter::once(graphic_pipeline.descriptor_set().raw()),
                &[],
            );

            // === Bind buffers
            let buffers = vertex_buffers
                .iter()
                .map(|buffer| (buffer.borrow_buffer(), SubRange::WHOLE))
                .collect::<Vec<_>>();
            self.command_buffer.bind_vertex_buffers(0, buffers);
            self.command_buffer.bind_index_buffer(IndexBufferView {
                buffer: index_buffer.buffer(),
                range: SubRange::WHOLE,
                index_type: gfx_hal::IndexType::U32,
            });

            // === Draw
            let (render_pass, clear_values) = if self.context.use_first_render_pass {
                self.context.use_first_render_pass = false;
                (
                    self.context.first_render_pass.deref(),
                    vec![
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
                )
            } else {
                (self.context.render_pass.deref(), vec![])
            };
            self.command_buffer.begin_render_pass(
                render_pass,
                self.frame_buffer,
                self.context.display_size.viewport.rect,
                &clear_values,
                SubpassContents::Inline,
            );
            self.command_buffer
                .draw_indexed(0..index_size as u32, 0, 0..1);
            self.command_buffer.end_render_pass();
        }
    }

    pub fn display_size(&self) -> &DisplaySize {
        &self.context.display_size
    }

    pub fn write_descriptor_sets(&self, write_descriptor_sets: WriteDescriptorSetsCommon<B>) {
        unsafe {
            self.context
                .device
                .write_descriptor_sets(write_descriptor_sets.get())
        }
    }
}
