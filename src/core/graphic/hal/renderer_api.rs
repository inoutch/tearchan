use crate::core::graphic::hal::graphic_pipeline::GraphicPipeline;
use crate::core::graphic::hal::image::Image;
use crate::core::graphic::hal::shader::Shader;
use crate::core::graphic::hal::texture::Texture;
use crate::core::graphic::hal::uniform_buffer::UniformBuffer;
use crate::core::graphic::hal::vertex_buffer::VertexBuffer;
use crate::core::graphic::shader::attribute::Attribute;
use crate::core::graphic::shader::shader_program::{ShaderProgram, ShaderProgramCommon};
use crate::core::graphic::shader::shader_source::ShaderSource;
use gfx_hal::adapter::MemoryType;
use gfx_hal::command::CommandBuffer;
use gfx_hal::device::Device;
use gfx_hal::pso::Descriptor;
use gfx_hal::queue::QueueGroup;
use gfx_hal::Limits;
use nalgebra_glm::Vec2;
use std::rc::Rc;

pub struct Api<'a, B: gfx_hal::Backend> {
    device: &'a Rc<B::Device>,
    memory_types: &'a [MemoryType],
    limits: &'a Limits,
    command_pool: &'a mut B::CommandPool,
    command_buffer: &'a mut B::CommandBuffer,
    queue_group: &'a mut QueueGroup<B>,
    render_pass: &'a B::RenderPass,
    frame_buffer: &'a B::Framebuffer,
    viewport: &'a gfx_hal::pso::Viewport,
    screen_size: &'a Vec2,
}

impl<'a, B: gfx_hal::Backend> Api<'a, B> {
    pub fn new(
        device: &'a Rc<B::Device>,
        limits: &'a Limits,
        memory_types: &'a [MemoryType],
        command_pool: &'a mut B::CommandPool,
        command_buffer: &'a mut B::CommandBuffer,
        queue_group: &'a mut QueueGroup<B>,
        render_pass: &'a B::RenderPass,
        frame_buffer: &'a B::Framebuffer,
        viewport: &'a gfx_hal::pso::Viewport,
        screen_size: &'a Vec2,
    ) -> Api<'a, B> {
        Api {
            memory_types,
            limits,
            device,
            command_pool,
            command_buffer,
            queue_group,
            render_pass,
            frame_buffer,
            viewport,
            screen_size,
        }
    }

    pub fn create_vertex_buffer(&self, vertices: &[f32]) -> VertexBuffer<B> {
        VertexBuffer::new(self.device, &self.memory_types, vertices)
    }

    pub fn create_uniform_buffer<T>(&self, data_source: &[T]) -> UniformBuffer<B, T> {
        UniformBuffer::new(self.device, self.memory_types, data_source)
    }

    pub fn create_shader(
        &self,
        shader_source: ShaderSource,
        attributes: Vec<Attribute>,
        descriptor_sets: Vec<gfx_hal::pso::DescriptorSetLayoutBinding>,
    ) -> Shader<B> {
        debug_assert!(!attributes.is_empty(), "attributes are empty");
        Shader::new(&self.device, shader_source, attributes, descriptor_sets)
    }

    pub fn create_shader_program(&self, shader: Shader<B>) -> ShaderProgramCommon<B> {
        ShaderProgramCommon::new(self.device, self.memory_types, shader)
    }

    pub fn create_texture(&mut self, image: &Image) -> Texture<B> {
        Texture::new(
            self.device,
            self.command_pool,
            self.queue_group,
            self.memory_types,
            self.limits,
            image,
        )
    }

    pub fn create_graphic_pipeline(&mut self, shader: &Shader<B>) -> GraphicPipeline<B> {
        GraphicPipeline::new(&self.device, self.render_pass, shader)
    }

    pub fn draw_triangle(
        &mut self,
        graphic_pipeline: &GraphicPipeline<B>,
        vertex_buffers: &Vec<&VertexBuffer<B>>,
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
                self.render_pass,
                self.frame_buffer,
                self.viewport.rect,
                &[gfx_hal::command::ClearValue {
                    color: gfx_hal::command::ClearColor {
                        float32: [0.3, 0.3, 0.3, 1.0],
                    },
                }],
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
        unsafe { self.device.write_descriptor_sets(write_descriptor_sets) }
    }

    pub fn device(&self) -> Rc<B::Device> {
        Rc::clone(self.device)
    }

    pub fn limits(&self) -> &Limits {
        self.limits
    }
}
