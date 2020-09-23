use crate::batch::batch_buffer::BatchBuffer;
use crate::hal::buffer::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
use crate::hal::buffer::index_buffer::IndexBufferCommon;
use crate::hal::buffer::vertex_buffer::VertexBufferCommon;
use crate::hal::render_bundle::RenderBundleCommon;
use gfx_hal::adapter::MemoryType;
use gfx_hal::memory::{Properties, Requirements};
use gfx_hal::{Backend, MemoryTypeId};

pub fn find_memory_type(
    memory_types: &[MemoryType],
    buffer_req: &Requirements,
    properties: Properties,
) -> MemoryTypeId {
    memory_types
        .iter()
        .enumerate()
        .position(|(id, memory_type)| {
            buffer_req.type_mask & (1 << id) as u32 != 0
                && memory_type.properties.contains(properties)
        })
        .unwrap()
        .into()
}

const DEFAULT_INDEX_BUFFER_SIZE: usize = 1000;
const DEFAULT_VERTEX_BUFFER_SIZE: usize = 1000;

pub fn create_index_batch_buffer<B: Backend>(
    render_bundle: &RenderBundleCommon<B>,
) -> BatchBuffer<IndexBufferCommon<B>> {
    // TODO: Improve performance
    BatchBuffer::new(
        IndexBufferCommon::new(render_bundle, &[0; DEFAULT_INDEX_BUFFER_SIZE]),
        |buffer, size| {
            let new_buffer = buffer
                .create_index_buffer(&vec![0; size])
                .expect("device is dropped");
            let src = buffer.open(0, buffer.len());
            let mut dst = new_buffer.open(0, buffer.len());

            for i in 0..buffer.len() {
                dst.set(src.get(i), i);
            }
            buffer.close(src);
            new_buffer.close(dst);
            new_buffer
        },
    )
}

pub fn create_vertex_batch_buffer<B: Backend>(
    render_bundle: &RenderBundleCommon<B>,
) -> BatchBuffer<VertexBufferCommon<B>> {
    // TODO: Improve performance
    BatchBuffer::new(
        VertexBufferCommon::new(render_bundle, &[0.0f32; DEFAULT_VERTEX_BUFFER_SIZE]),
        |buffer, size| {
            let new_buffer = buffer
                .create_vertex_buffer(&vec![0.0f32; size])
                .expect("device is dropped");
            let src = buffer.open(0, buffer.len());
            let mut dst = new_buffer.open(0, buffer.len());

            for i in 0..buffer.len() {
                dst.set(src.get(i), i);
            }
            buffer.close(src);
            new_buffer.close(dst);
            new_buffer
        },
    )
}
