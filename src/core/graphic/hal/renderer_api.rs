use crate::core::graphic::hal::graphic_pipeline::{GraphicPipelineCommon, GraphicPipelineConfig};
use crate::core::graphic::hal::index_buffer::IndexBufferCommon;
use crate::core::graphic::hal::renderer::{RendererApiContext, RendererApiProperties};
use crate::core::graphic::hal::shader::attribute::Attribute;
use crate::core::graphic::hal::shader::shader_source::ShaderSource;
use crate::core::graphic::hal::shader::ShaderCommon;
use crate::core::graphic::hal::texture::{TextureCommon, TextureConfig};
use crate::core::graphic::hal::uniform_buffer::UniformBufferCommon;
use crate::core::graphic::hal::vertex_buffer::VertexBufferCommon;
use crate::core::graphic::hal::write_descriptor_sets::WriteDescriptorSetsCommon;
use crate::core::graphic::image::Image;
use crate::math::mesh::IndexType;
use gfx_hal::buffer::{IndexBufferView, SubRange};
use gfx_hal::command::{ClearColor, ClearDepthStencil, ClearValue, CommandBuffer, SubpassContents};
use gfx_hal::device::Device;
use nalgebra_glm::{vec2, Vec2};
use std::ops::Deref;

pub struct RendererApiCommon<'a, B: gfx_hal::Backend> {
    context: &'a mut RendererApiContext<B>,
    static_properties: &'a RendererApiProperties,
    command_pool: &'a mut B::CommandPool,
    command_buffer: &'a mut B::CommandBuffer,
    frame_buffer: &'a B::Framebuffer,
    screen_size: Vec2,
    window_size: Vec2,
}

impl<'a, B: gfx_hal::Backend> RendererApiCommon<'a, B> {
    pub fn new(
        context: &'a mut RendererApiContext<B>,
        static_properties: &'a RendererApiProperties,
        command_pool: &'a mut B::CommandPool,
        command_buffer: &'a mut B::CommandBuffer,
        frame_buffer: &'a B::Framebuffer,
    ) -> RendererApiCommon<'a, B> {
        let screen_size = vec2(
            static_properties.viewport.rect.w as f32,
            static_properties.viewport.rect.h as f32,
        );
        let window_size = vec2(
            static_properties.dimensions.width as f32,
            static_properties.dimensions.height as f32,
        );
        RendererApiCommon {
            context,
            static_properties,
            command_pool,
            command_buffer,
            frame_buffer,
            screen_size,
            window_size,
        }
    }

    pub fn create_vertex_buffer(&self, vertices: &[f32]) -> VertexBufferCommon<B> {
        VertexBufferCommon::new(
            &self.context.device,
            &self.static_properties.memory_types,
            vertices,
        )
    }

    pub fn create_index_buffer(&self, indices: &[IndexType]) -> IndexBufferCommon<B> {
        IndexBufferCommon::new(
            &self.context.device,
            &self.static_properties.memory_types,
            indices,
        )
    }

    pub fn create_uniform_buffer<T>(&self, data_source: &[T]) -> UniformBufferCommon<B, T> {
        UniformBufferCommon::new(
            &self.context.device,
            &self.static_properties.memory_types,
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
            &self.static_properties.memory_types,
            &self.static_properties.limits,
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
                    self.static_properties.viewport.rect,
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
                    self.static_properties.viewport.rect,
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
                self.static_properties.viewport.rect,
                &clear_values,
                SubpassContents::Inline,
            );
            self.command_buffer
                .draw_indexed(0..index_size as u32, 0, 0..1);
            self.command_buffer.end_render_pass();
        }
    }

    pub fn screen_size(&self) -> &Vec2 {
        &self.screen_size
    }

    pub fn window_size(&self) -> &Vec2 {
        &self.window_size
    }

    pub fn write_descriptor_sets(&self, write_descriptor_sets: WriteDescriptorSetsCommon<B>) {
        unsafe {
            self.context
                .device
                .write_descriptor_sets(write_descriptor_sets.get())
        }
    }
}
