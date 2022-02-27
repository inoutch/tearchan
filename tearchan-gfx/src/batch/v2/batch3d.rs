use crate::batch::v2::buffer::BatchBuffer;
use crate::batch::v2::context::BatchContext;
use crate::batch::v2::provider::BatchProvider;
use crate::batch::v2::{Batch, BatchEvent};
use crate::v2::buffer::Buffer;
use nalgebra_glm::{Vec2, Vec3, Vec4};
use std::ops::{Deref, DerefMut};
use wgpu::RenderPass;

pub const BATCH3D_ATTRIBUTE_POSITION: u32 = 0;
pub const BATCH3D_ATTRIBUTE_TEXCOORD: u32 = 1;
pub const BATCH3D_ATTRIBUTE_COLOR: u32 = 2;
pub const BATCH3D_ATTRIBUTE_NORMAL: u32 = 3;

pub struct Batch3D {
    batch: Batch<Batch3DProvider>,
}

impl Deref for Batch3D {
    type Target = Batch<Batch3DProvider>;

    fn deref(&self) -> &Self::Target {
        &self.batch
    }
}

impl DerefMut for Batch3D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.batch
    }
}

impl Batch3D {
    pub fn new(device: &wgpu::Device) -> Self {
        let len: usize = 1024 * 100;
        let index_usage =
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
        let vertex_usage = wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC;
        let provider = Batch3DProvider {
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
            normal_buffer: BatchBuffer::new(Buffer::new(
                device,
                len,
                "NormalBuffer".to_string(),
                vertex_usage,
            )),
        };

        Batch3D {
            batch: Batch::new(provider, len, len),
        }
    }

    pub fn bind<'a>(&'a self, rpass: &mut RenderPass<'a>) {
        rpass.set_index_buffer(
            self.provider.index_buffer.buffer().slice(..),
            wgpu::IndexFormat::Uint32,
        );
        rpass.set_vertex_buffer(
            BATCH3D_ATTRIBUTE_POSITION as u32,
            self.provider.position_buffer.buffer().slice(..),
        );
        rpass.set_vertex_buffer(
            BATCH3D_ATTRIBUTE_TEXCOORD as u32,
            self.provider.texcoord_buffer.buffer().slice(..),
        );
        rpass.set_vertex_buffer(
            BATCH3D_ATTRIBUTE_COLOR as u32,
            self.provider.color_buffer.buffer().slice(..),
        );
        rpass.set_vertex_buffer(
            BATCH3D_ATTRIBUTE_NORMAL as u32,
            self.provider.normal_buffer.buffer().slice(..),
        );
    }
}

pub struct Batch3DProvider {
    index_buffer: BatchBuffer<Buffer<u32>, u32>,
    position_buffer: BatchBuffer<Buffer<Vec3>, Vec3>,
    texcoord_buffer: BatchBuffer<Buffer<Vec2>, Vec2>,
    color_buffer: BatchBuffer<Buffer<Vec4>, Vec4>,
    normal_buffer: BatchBuffer<Buffer<Vec3>, Vec3>,
}

impl<'a> BatchProvider<'a> for Batch3DProvider {
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
                    BATCH3D_ATTRIBUTE_POSITION => {
                        self.position_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v3f32_vertices(attribute).unwrap(),
                        );
                    }
                    BATCH3D_ATTRIBUTE_TEXCOORD => {
                        self.texcoord_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v2f32_vertices(attribute).unwrap(),
                        );
                    }
                    BATCH3D_ATTRIBUTE_COLOR => {
                        self.color_buffer.write(
                            context.writer(),
                            pointer,
                            &object.get_v4f32_vertices(attribute).unwrap(),
                        );
                    }
                    BATCH3D_ATTRIBUTE_NORMAL => {
                        self.normal_buffer.write(
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
                self.normal_buffer.clear(context.writer(), pointer);
            }
            BatchEvent::ResizeIndexBuffer { len } => {
                self.index_buffer.resize(context.resizer(), len);
            }
            BatchEvent::ResizeVertexBuffer { len } => {
                self.position_buffer.resize(context.resizer(), len);
                self.texcoord_buffer.resize(context.resizer(), len);
                self.color_buffer.resize(context.resizer(), len);
                self.normal_buffer.resize(context.resizer(), len);
            }
            _ => {}
        }
    }
}
