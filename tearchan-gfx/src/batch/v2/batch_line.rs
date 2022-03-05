use crate::batch::v2::buffer::BatchBuffer;
use crate::batch::v2::context::BatchContext;
use crate::batch::v2::provider::BatchProvider;
use crate::batch::v2::{Batch, BatchEvent};
use crate::v2::buffer::Buffer;
use nalgebra_glm::{Vec3, Vec4};
use std::ops::{Deref, DerefMut};
use wgpu::Device;

pub const BATCH_LINE_ATTRIBUTE_POSITION: u32 = 0;
pub const BATCH_LINE_ATTRIBUTE_COLOR: u32 = 1;

pub struct BatchLine {
    batch: Batch<BatchLineProvider>,
}

impl BatchLine {
    pub fn new(device: &Device) -> Self {
        let len = 102400usize;
        let index_usage =
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
        let vertex_usage = wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC;
        let provider = BatchLineProvider {
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
            color_buffer: BatchBuffer::new(Buffer::new(
                device,
                len,
                "ColorBuffer".to_string(),
                vertex_usage,
            )),
        };
        BatchLine {
            batch: Batch::new(provider, len, len),
        }
    }

    pub fn bind<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_index_buffer(
            self.provider.index_buffer.buffer().slice(..),
            wgpu::IndexFormat::Uint32,
        );
        rpass.set_vertex_buffer(
            BATCH_LINE_ATTRIBUTE_POSITION,
            self.provider.position_buffer.buffer().slice(..),
        );
        rpass.set_vertex_buffer(
            BATCH_LINE_ATTRIBUTE_COLOR,
            self.provider.color_buffer.buffer().slice(..),
        );
    }
}

impl Deref for BatchLine {
    type Target = Batch<BatchLineProvider>;

    fn deref(&self) -> &Self::Target {
        &self.batch
    }
}

impl DerefMut for BatchLine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.batch
    }
}

pub struct BatchLineProvider {
    index_buffer: BatchBuffer<Buffer<u32>, u32>,
    position_buffer: BatchBuffer<Buffer<Vec3>, Vec3>,
    color_buffer: BatchBuffer<Buffer<Vec4>, Vec4>,
}

impl<'a> BatchProvider<'a> for BatchLineProvider {
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
                    BATCH_LINE_ATTRIBUTE_POSITION => {
                        self.position_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v3f32_vertices(attribute).unwrap(),
                        );
                    }
                    BATCH_LINE_ATTRIBUTE_COLOR => {
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
                self.color_buffer.clear(context.writer(), pointer);
            }
            BatchEvent::ResizeIndexBuffer { len } => {
                self.index_buffer.resize(context.resizer(), len);
            }
            BatchEvent::ResizeVertexBuffer { len } => {
                self.position_buffer.resize(context.resizer(), len);
                self.color_buffer.resize(context.resizer(), len);
            }
            _ => {}
        }
    }
}
