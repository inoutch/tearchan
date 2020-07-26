use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::hal::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
use crate::core::graphic::hal::index_buffer::IndexBufferCommon;
use crate::core::graphic::hal::graphics::GraphicsCommon;
use crate::core::graphic::hal::vertex_buffer::VertexBufferCommon;
use gfx_hal::Backend;

const DEFAULT_INDEX_BUFFER_SIZE: usize = 1000;
const DEFAULT_VERTEX_BUFFER_SIZE: usize = 1000;

pub fn create_index_batch_buffer<B: Backend>(
    api: &GraphicsCommon<B>,
) -> BatchBuffer<IndexBufferCommon<B>> {
    // TODO: Improve performance
    BatchBuffer::new(
        api.create_index_buffer(&[0; DEFAULT_INDEX_BUFFER_SIZE]),
        |buffer, size| {
            let new_buffer = buffer
                .create_index_buffer(&vec![0; size])
                .expect("device is dropped");
            let src = buffer.open(0, buffer.size());
            let mut dst = new_buffer.open(0, buffer.size());

            for i in 0..buffer.size() {
                dst.set(src.get(i), i);
            }
            buffer.close(src);
            new_buffer.close(dst);
            new_buffer
        },
    )
}

pub fn create_vertex_batch_buffer<B: Backend>(
    api: &GraphicsCommon<B>,
) -> BatchBuffer<VertexBufferCommon<B>> {
    // TODO: Improve performance
    BatchBuffer::new(
        api.create_vertex_buffer(&[0.0f32; DEFAULT_VERTEX_BUFFER_SIZE]),
        |buffer, size| {
            let new_buffer = buffer
                .create_vertex_buffer(&vec![0.0f32; size])
                .expect("device is dropped");
            let src = buffer.open(0, buffer.size());
            let mut dst = new_buffer.open(0, buffer.size());

            for i in 0..buffer.size() {
                dst.set(src.get(i), i);
            }
            buffer.close(src);
            new_buffer.close(dst);
            new_buffer
        },
    )
}
