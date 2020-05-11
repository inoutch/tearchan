use crate::core::graphic::hal::graphic_pipeline::GraphicPipelineCommon;
use crate::core::graphic::hal::image::Image;
use crate::core::graphic::hal::renderer::{RendererApiContext, RendererApiStaticContext};
use crate::core::graphic::hal::shader::Shader;
use crate::core::graphic::hal::texture::TextureCommon;
use crate::core::graphic::hal::uniform_buffer::UniformBufferCommon;
use crate::core::graphic::hal::vertex_buffer::VertexBufferCommon;
use crate::core::graphic::shader::attribute::Attribute;
use crate::core::graphic::shader::shader_program::ShaderProgramCommon;
use crate::core::graphic::shader::shader_source::ShaderSource;
use gfx_hal::command::{ClearDepthStencil, CommandBuffer};
use gfx_hal::device::Device;
use gfx_hal::pso::Descriptor;
use nalgebra_glm::Vec2;
use std::ops::Deref;

pub struct RendererApiCommon<'a, B: gfx_hal::Backend> {
    context: &'a mut RendererApiContext<B>,
    static_context: &'a RendererApiStaticContext,
    command_pool: &'a mut B::CommandPool,
    command_buffer: &'a mut B::CommandBuffer,
    frame_buffer: &'a B::Framebuffer,
    viewport: &'a gfx_hal::pso::Viewport,
    screen_size: &'a Vec2,
}

impl<'a, B: gfx_hal::Backend> RendererApiCommon<'a, B> {
    pub fn new(
        context: &'a mut RendererApiContext<B>,
        static_context: &'a RendererApiStaticContext,
        command_pool: &'a mut B::CommandPool,
        command_buffer: &'a mut B::CommandBuffer,
        frame_buffer: &'a B::Framebuffer,
        viewport: &'a gfx_hal::pso::Viewport,
        screen_size: &'a Vec2,
    ) -> RendererApiCommon<'a, B> {
        RendererApiCommon {
            context,
            static_context,
            command_pool,
            command_buffer,
            frame_buffer,
            viewport,
            screen_size,
        }
    }

    pub fn create_vertex_buffer(&self, vertices: &[f32]) -> VertexBufferCommon<B> {
        VertexBufferCommon::new(
            &self.context.device,
            &self.static_context.memory_types,
            vertices,
        )
    }

    pub fn create_uniform_buffer<T>(&self, data_source: &[T]) -> UniformBufferCommon<B, T> {
        UniformBufferCommon::new(
            &self.context.device,
            &self.static_context.memory_types,
            data_source,
        )
    }

    pub fn create_shader(
        &self,
        shader_source: ShaderSource,
        attributes: Vec<Attribute>,
        descriptor_sets: Vec<gfx_hal::pso::DescriptorSetLayoutBinding>,
    ) -> Shader<B> {
        debug_assert!(!attributes.is_empty(), "attributes are empty");
        Shader::new(
            &self.context.device,
            shader_source,
            attributes,
            descriptor_sets,
        )
    }

    pub fn create_shader_program(&self, shader: Shader<B>) -> ShaderProgramCommon<B> {
        ShaderProgramCommon::new(
            &self.context.device,
            &self.static_context.memory_types,
            shader,
        )
    }

    pub fn create_texture(&mut self, image: &Image) -> TextureCommon<B> {
        TextureCommon::new(
            &self.context.device,
            self.command_pool,
            &mut self.context.queue_group,
            &self.static_context.memory_types,
            &self.static_context.limits,
            image,
        )
    }

    pub fn create_graphic_pipeline(&mut self, shader: &Shader<B>) -> GraphicPipelineCommon<B> {
        GraphicPipelineCommon::new(
            &self.context.device,
            self.context.render_pass.deref(),
            shader,
        )
    }

    pub fn draw_triangle(
        &mut self,
        graphic_pipeline: &GraphicPipelineCommon<B>,
        vertex_buffers: &[&VertexBufferCommon<B>],
        triangle_count: usize,
    ) {
        unsafe {
            self.command_buffer
                .bind_graphics_pipeline(graphic_pipeline.borrow_pipeline());
            self.command_buffer.bind_graphics_descriptor_sets(
                graphic_pipeline.borrow_pipeline_layout(),
                0,
                std::iter::once(graphic_pipeline.borrow_descriptor_set()),
                &[],
            );

            let buffers: Vec<(&B::Buffer, gfx_hal::buffer::SubRange)> = vertex_buffers
                .iter()
                .map(|x| (x.borrow_buffer(), gfx_hal::buffer::SubRange::WHOLE))
                .collect();
            self.command_buffer.bind_vertex_buffers(0, buffers);
            self.command_buffer.begin_render_pass(
                self.context.render_pass.deref(),
                self.frame_buffer,
                self.viewport.rect,
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
            self.command_buffer.draw(0..triangle_count as u32, 0..1);
            self.command_buffer.end_render_pass();
        }
    }

    pub fn screen_size(&self) -> &Vec2 {
        self.screen_size
    }

    pub fn write_descriptor_sets(
        &self,
        write_descriptor_sets: Vec<
            gfx_hal::pso::DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>>,
        >,
    ) {
        unsafe {
            self.context
                .device
                .write_descriptor_sets(write_descriptor_sets)
        }
    }
}
