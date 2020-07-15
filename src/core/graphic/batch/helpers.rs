use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::hal::index_buffer::IndexBufferCommon;
use crate::core::graphic::hal::renderer_api::RendererApiCommon;
use crate::core::graphic::hal::vertex_buffer::VertexBufferCommon;
use gfx_hal::Backend;

const DEFAULT_INDEX_BUFFER_SIZE: usize = 1000;
const DEFAULT_VERTEX_BUFFER_SIZE: usize = 1000;

pub fn create_index_batch_buffer<B: Backend>(
    api: &RendererApiCommon<B>,
) -> BatchBuffer<IndexBufferCommon<B>> {
    // TODO: Let the buffer be copied
    BatchBuffer::new(
        api.create_index_buffer(&[0; DEFAULT_INDEX_BUFFER_SIZE]),
        |buffer, size| {
            buffer
                .create_index_buffer(&vec![0; size])
                .expect("device is dropped")
        },
    )
}

pub fn create_vertex_batch_buffer<B: Backend>(
    api: &RendererApiCommon<B>,
) -> BatchBuffer<VertexBufferCommon<B>> {
    // TODO: Let the buffer be copied
    BatchBuffer::new(
        api.create_vertex_buffer(&[0.0f32; DEFAULT_VERTEX_BUFFER_SIZE]),
        |buffer, size| {
            buffer
                .create_vertex_buffer(&vec![0.0f32; size])
                .expect("device is dropped")
        },
    )
}
