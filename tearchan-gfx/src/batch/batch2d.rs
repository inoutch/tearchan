use crate::batch::buffer::BatchBuffer;
use crate::batch::context::BatchContext;
use crate::batch::provider::BatchProvider;
use crate::batch::{Batch, BatchEvent};
use crate::buffer::Buffer;
use nalgebra_glm::{Vec2, Vec3, Vec4};
use std::ops::{Deref, DerefMut};
use wgpu::RenderPass;

pub const BATCH2D_ATTRIBUTE_POSITION: u32 = 0;
pub const BATCH2D_ATTRIBUTE_TEXCOORD: u32 = 1;
pub const BATCH2D_ATTRIBUTE_COLOR: u32 = 2;

pub struct Batch2D {
    batch: Batch<Batch2DProvider>,
}

impl Deref for Batch2D {
    type Target = Batch<Batch2DProvider>;

    fn deref(&self) -> &Self::Target {
        &self.batch
    }
}

impl DerefMut for Batch2D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.batch
    }
}

impl Batch2D {
    pub fn new(device: &wgpu::Device) -> Self {
        let len: usize = 1024 * 100;
        let index_usage =
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
        let vertex_usage = wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC;
        let provider = Batch2DProvider {
            index_buffer: BatchBuffer::new(Buffer::new(
                device,
                len,
                "IndexBuffer".to_string(),
                index_usage,
            )),
            position_buffer: BatchBuffer::new(Buffer::new(
                device,
                len,
                "PositionBuffer".to_string(),
                vertex_usage,
            )),
            texcoord_buffer: BatchBuffer::new(Buffer::new(
                device,
                len,
                "PositionBuffer".to_string(),
                vertex_usage,
            )),
            color_buffer: BatchBuffer::new(Buffer::new(
                device,
                len,
                "PositionBuffer".to_string(),
                vertex_usage,
            )),
        };

        Batch2D {
            batch: Batch::new(provider, len, len),
        }
    }

    pub fn bind<'a>(&'a self, rpass: &mut RenderPass<'a>) {
        rpass.set_index_buffer(
            self.provider.index_buffer.buffer().slice(..),
            wgpu::IndexFormat::Uint32,
        );
        rpass.set_vertex_buffer(
            BATCH2D_ATTRIBUTE_POSITION as u32,
            self.provider.position_buffer.buffer().slice(..),
        );
        rpass.set_vertex_buffer(
            BATCH2D_ATTRIBUTE_TEXCOORD as u32,
            self.provider.texcoord_buffer.buffer().slice(..),
        );
        rpass.set_vertex_buffer(
            BATCH2D_ATTRIBUTE_COLOR as u32,
            self.provider.color_buffer.buffer().slice(..),
        );
    }
}

pub struct Batch2DProvider {
    index_buffer: BatchBuffer<Buffer<u32>, u32>,
    position_buffer: BatchBuffer<Buffer<Vec3>, Vec3>,
    texcoord_buffer: BatchBuffer<Buffer<Vec2>, Vec2>,
    color_buffer: BatchBuffer<Buffer<Vec4>, Vec4>,
}

impl<'a> BatchProvider<'a> for Batch2DProvider {
    type Context = BatchContext<'a>;

    fn run(&mut self, context: &mut Self::Context, event: BatchEvent) {
        match event {
            BatchEvent::WriteToIndexBuffer {
                pointer, object, ..
            } => {
                self.index_buffer.write(
                    context.writer(),
                    pointer,
                    &object.get_v1u32_indices().unwrap(),
                );
            }
            BatchEvent::WriteToVertexBuffer {
                pointer,
                object,
                attribute,
                ..
            } => {
                match attribute {
                    BATCH2D_ATTRIBUTE_POSITION => {
                        self.position_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v3f32_vertices(attribute).unwrap(),
                        );
                    }
                    BATCH2D_ATTRIBUTE_TEXCOORD => {
                        self.texcoord_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v2f32_vertices(attribute).unwrap(),
                        );
                    }
                    BATCH2D_ATTRIBUTE_COLOR => {
                        self.color_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v4f32_vertices(attribute).unwrap(),
                        );
                    }
                    _ => {}
                };
            }
            BatchEvent::ClearToIndexBuffer { pointer } => {
                self.index_buffer.clear(context.writer(), pointer);
            }
            BatchEvent::ClearToVertexBuffer { pointer } => {
                self.position_buffer.clear(context.writer(), pointer);
                self.texcoord_buffer.clear(context.writer(), pointer);
                self.color_buffer.clear(context.writer(), pointer);
            }
            BatchEvent::ResizeIndexBuffer { len } => {
                self.index_buffer.resize(context.resizer(), len);
            }
            BatchEvent::ResizeVertexBuffer { len } => {
                self.position_buffer.resize(context.resizer(), len);
                self.texcoord_buffer.resize(context.resizer(), len);
                self.color_buffer.resize(context.resizer(), len);
            }
            _ => {}
        }
    }
}
