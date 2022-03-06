use crate::batch::buffer::BatchBuffer;
use crate::batch::context::BatchContext;
use crate::batch::provider::BatchProvider;
use crate::batch::{Batch, BatchEvent};
use crate::buffer::Buffer;
use nalgebra_glm::{Vec2, Vec3, Vec4};
use std::ops::{Deref, DerefMut};

pub const BATCH_BILLBOARD_ATTRIBUTE_POSITION: u32 = 0;
pub const BATCH_BILLBOARD_ATTRIBUTE_TEXCOORD: u32 = 1;
pub const BATCH_BILLBOARD_ATTRIBUTE_COLOR: u32 = 2;
pub const BATCH_BILLBOARD_ATTRIBUTE_ORIGIN: u32 = 3;

pub struct BatchBillboard {
    batch: Batch<BatchBillboardProvider>,
}

impl BatchBillboard {
    pub fn new(device: &wgpu::Device) -> Self {
        let len = 102400usize;
        let index_usage =
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
        let vertex_usage = wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC;
        let provider = BatchBillboardProvider {
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
                "TexcoordBuffer".to_string(),
                vertex_usage,
            )),
            color_buffer: BatchBuffer::new(Buffer::new(
                device,
                len,
                "ColorBuffer".to_string(),
                vertex_usage,
            )),
            origin_buffer: BatchBuffer::new(Buffer::new(
                device,
                len,
                "OriginBuffer".to_string(),
                vertex_usage,
            )),
        };

        BatchBillboard {
            batch: Batch::new(provider, len, len),
        }
    }

    pub fn bind<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_index_buffer(
            self.provider.index_buffer.buffer().slice(..),
            wgpu::IndexFormat::Uint32,
        );
        rpass.set_vertex_buffer(
            BATCH_BILLBOARD_ATTRIBUTE_POSITION,
            self.provider.position_buffer.buffer().slice(..),
        );
        rpass.set_vertex_buffer(
            BATCH_BILLBOARD_ATTRIBUTE_TEXCOORD,
            self.provider.texcoord_buffer.buffer().slice(..),
        );
        rpass.set_vertex_buffer(
            BATCH_BILLBOARD_ATTRIBUTE_COLOR,
            self.provider.color_buffer.buffer().slice(..),
        );
        rpass.set_vertex_buffer(
            BATCH_BILLBOARD_ATTRIBUTE_ORIGIN,
            self.provider.origin_buffer.buffer().slice(..),
        );
    }
}

impl Deref for BatchBillboard {
    type Target = Batch<BatchBillboardProvider>;

    fn deref(&self) -> &Self::Target {
        &self.batch
    }
}

impl DerefMut for BatchBillboard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.batch
    }
}

pub struct BatchBillboardProvider {
    index_buffer: BatchBuffer<Buffer<u32>, u32>,
    position_buffer: BatchBuffer<Buffer<Vec3>, Vec3>,
    texcoord_buffer: BatchBuffer<Buffer<Vec2>, Vec2>,
    color_buffer: BatchBuffer<Buffer<Vec4>, Vec4>,
    origin_buffer: BatchBuffer<Buffer<Vec3>, Vec3>,
}

impl<'a> BatchProvider<'a> for BatchBillboardProvider {
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
                    BATCH_BILLBOARD_ATTRIBUTE_POSITION => {
                        self.position_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v3f32_vertices(attribute).unwrap(),
                        );
                    }
                    BATCH_BILLBOARD_ATTRIBUTE_TEXCOORD => {
                        self.texcoord_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v2f32_vertices(attribute).unwrap(),
                        );
                    }
                    BATCH_BILLBOARD_ATTRIBUTE_COLOR => {
                        self.color_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v4f32_vertices(attribute).unwrap(),
                        );
                    }
                    BATCH_BILLBOARD_ATTRIBUTE_ORIGIN => {
                        self.origin_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v3f32_vertices(attribute).unwrap(),
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
                self.origin_buffer.clear(context.writer(), pointer);
            }
            BatchEvent::ResizeIndexBuffer { len } => {
                self.index_buffer.resize(context.resizer(), len);
            }
            BatchEvent::ResizeVertexBuffer { len } => {
                self.position_buffer.resize(context.resizer(), len);
                self.texcoord_buffer.resize(context.resizer(), len);
                self.color_buffer.resize(context.resizer(), len);
                self.origin_buffer.resize(context.resizer(), len);
            }
            _ => {}
        }
    }
}
