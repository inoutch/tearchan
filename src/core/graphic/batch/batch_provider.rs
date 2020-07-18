use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::BatchContext;
use crate::core::graphic::hal::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
use crate::extension::shared::Shared;
use serde::export::fmt::Debug;
use std::rc::Rc;

pub struct BatchBufferContext<TBuffer> {
    pub buffer: TBuffer,
    pub stride: usize,
}

impl<TBuffer> BatchBufferContext<TBuffer> {
    pub fn new(buffer: TBuffer, stride: usize) -> Self {
        BatchBufferContext { buffer, stride }
    }
}

pub trait BatchProvider<TObject, TIndexBuffer: BufferInterface, TVertexBuffer: BufferInterface> {
    fn update(&mut self, context: &Rc<BatchContext<TObject>>, force: bool);

    fn index_buffer(&self) -> &BatchBuffer<TIndexBuffer>;

    fn index_buffer_mut(&mut self) -> &mut BatchBuffer<TIndexBuffer>;

    fn index_size(&self, object: &Shared<TObject>) -> usize;

    fn vertex_buffer_contexts_mut(
        &mut self,
    ) -> &mut Vec<BatchBufferContext<BatchBuffer<TVertexBuffer>>>;

    fn vertex_buffer_contexts(&self) -> &Vec<BatchBufferContext<BatchBuffer<TVertexBuffer>>>;

    fn vertex_size(&self, object: &Shared<TObject>) -> usize;

    fn open(&mut self);

    fn close(&mut self);
}

pub fn open_buffers<
    TIndexBufferDataType,
    TIndexBufferMappedMemory,
    TIndexBuffer,
    TVertexBufferDataType,
    TVertexBufferMappedMemory,
    TVertexBuffer,
>(
    index_buffer: &BatchBuffer<TIndexBuffer>,
    index_mapping: &mut Option<TIndexBufferMappedMemory>,
    vertex_buffer_contexts: &[BatchBufferContext<BatchBuffer<TVertexBuffer>>],
    vertex_mappings: &mut Vec<TVertexBufferMappedMemory>,
) where
    TIndexBufferDataType: Clone + Debug,
    TIndexBufferMappedMemory: BufferMappedMemoryInterface<TIndexBufferDataType>,
    TIndexBuffer: BufferInterface<
        DataType = TIndexBufferDataType,
        MappedMemoryType = TIndexBufferMappedMemory,
    >,
    TVertexBufferDataType: Clone + Debug,
    TVertexBufferMappedMemory: BufferMappedMemoryInterface<TVertexBufferDataType>,
    TVertexBuffer: BufferInterface<
        DataType = TVertexBufferDataType,
        MappedMemoryType = TVertexBufferMappedMemory,
    >,
{
    *index_mapping = Some(index_buffer.buffer().open(0, index_buffer.size()));
    *vertex_mappings = vertex_buffer_contexts
        .iter()
        .map(|context| context.buffer.buffer().open(0, context.buffer.size()))
        .collect();
}

pub fn close_buffers<
    TIndexBufferDataType,
    TIndexBufferMappedMemory,
    TIndexBuffer,
    TVertexBufferDataType,
    TVertexBufferMappedMemory,
    TVertexBuffer,
>(
    index_buffer: &BatchBuffer<TIndexBuffer>,
    index_mapping: &mut Option<TIndexBufferMappedMemory>,
    vertex_buffer_contexts: &[BatchBufferContext<BatchBuffer<TVertexBuffer>>],
    vertex_mappings: &mut Vec<TVertexBufferMappedMemory>,
) where
    TIndexBufferDataType: Clone + Debug,
    TIndexBufferMappedMemory: BufferMappedMemoryInterface<TIndexBufferDataType>,
    TIndexBuffer: BufferInterface<
        DataType = TIndexBufferDataType,
        MappedMemoryType = TIndexBufferMappedMemory,
    >,
    TVertexBufferDataType: Clone + Debug,
    TVertexBufferMappedMemory: BufferMappedMemoryInterface<TVertexBufferDataType>,
    TVertexBuffer: BufferInterface<
        DataType = TVertexBufferDataType,
        MappedMemoryType = TVertexBufferMappedMemory,
    >,
{
    let index_mapping = index_mapping.take().unwrap();
    index_buffer.buffer().close(index_mapping);

    let mut index = 0;
    while !vertex_mappings.is_empty() {
        let vertex_mapping = vertex_mappings.remove(0);
        vertex_buffer_contexts[index]
            .buffer
            .buffer()
            .close(vertex_mapping);
        index += 1;
    }
}

#[cfg(test)]
pub mod test {
    use crate::core::graphic::batch::batch_buffer::BatchBuffer;
    use crate::core::graphic::batch::batch_provider::{
        close_buffers, open_buffers, BatchBufferContext, BatchProvider,
    };
    use crate::core::graphic::batch::BatchContext;
    use crate::core::graphic::hal::index_buffer::test::{
        MockIndexBuffer, MockIndexBufferMappedMemory,
    };
    use crate::core::graphic::hal::vertex_buffer::test::{
        MockVertexBuffer, MockVertexBufferMappedMemory,
    };
    use crate::core::graphic::polygon::Polygon;
    use crate::extension::shared::Shared;
    use crate::math::mesh::IndexType;
    use crate::utility::test::func::MockFunc;
    use std::rc::Rc;

    pub struct MockBatchProvider {
        pub index_buffer: BatchBuffer<MockIndexBuffer>,
        pub index_mapping: Option<MockIndexBufferMappedMemory>,
        pub vertex_buffers: Vec<BatchBufferContext<BatchBuffer<MockVertexBuffer>>>,
        pub vertex_mappings: Vec<MockVertexBufferMappedMemory>,
    }

    impl MockBatchProvider {
        pub fn new(mock: &Shared<MockFunc>) -> Self {
            let indices = vec![0u32, 30];
            let factory = |buffer: &MockVertexBuffer, size| {
                let mut indices = buffer.vertices.borrow().to_owned();
                indices.resize(size, 0.0f32);
                MockVertexBuffer::new(&buffer.mock, &indices)
            };
            let positions =
                BatchBuffer::new(MockVertexBuffer::new(mock, &[0.0f32; 3 * 50]), factory);
            let colors = BatchBuffer::new(MockVertexBuffer::new(mock, &[0.0f32; 4 * 50]), factory);
            let texcoords =
                BatchBuffer::new(MockVertexBuffer::new(mock, &[0.0f32; 2 * 50]), factory);
            let normals = BatchBuffer::new(MockVertexBuffer::new(mock, &[0.0f32; 3 * 50]), factory);

            MockBatchProvider {
                index_buffer: BatchBuffer::new(
                    MockIndexBuffer::new(mock, &indices),
                    |buffer, size| {
                        let mut indices = buffer.indices.borrow().to_owned();
                        indices.resize(size, 0u32);
                        MockIndexBuffer::new(&buffer.mock, &indices)
                    },
                ),
                index_mapping: None,
                vertex_buffers: vec![
                    BatchBufferContext::new(positions, 3),
                    BatchBufferContext::new(colors, 4),
                    BatchBufferContext::new(texcoords, 2),
                    BatchBufferContext::new(normals, 3),
                ],
                vertex_mappings: vec![],
            }
        }
    }

    impl BatchProvider<Polygon, MockIndexBuffer, MockVertexBuffer> for MockBatchProvider {
        fn update(&mut self, context: &Rc<BatchContext<Polygon>>, force: bool) {
            // update positions, colors, texcoords, normals, indices
            let mut object = context.object.borrow_mut();
            let index_mapping = match &mut self.index_mapping {
                Some(mapping) => mapping,
                None => return,
            };
            object.copy_indices_into(
                index_mapping,
                context.index_pointer.borrow().first,
                (context.vertex_pointers[0].borrow().first / self.vertex_buffers[0].stride)
                    as IndexType,
                force,
            );
            object.copy_positions_into(
                &mut self.vertex_mappings[0],
                context.vertex_pointers[0].borrow().first,
                force,
            );
            object.copy_colors_into(
                &mut self.vertex_mappings[1],
                context.vertex_pointers[1].borrow().first,
                force,
            );
            object.copy_texcoords_into(
                &mut self.vertex_mappings[2],
                context.vertex_pointers[2].borrow().first,
                force,
            );
            object.copy_normals_into(
                &mut self.vertex_mappings[3],
                context.vertex_pointers[3].borrow().first,
                force,
            );
        }

        fn index_buffer(&self) -> &BatchBuffer<MockIndexBuffer> {
            &self.index_buffer
        }

        fn index_buffer_mut(&mut self) -> &mut BatchBuffer<MockIndexBuffer> {
            &mut self.index_buffer
        }

        fn index_size(&self, object: &Shared<Polygon>) -> usize {
            object.borrow().index_size()
        }

        fn vertex_buffer_contexts_mut(
            &mut self,
        ) -> &mut Vec<BatchBufferContext<BatchBuffer<MockVertexBuffer>>> {
            &mut self.vertex_buffers
        }

        fn vertex_buffer_contexts(
            &self,
        ) -> &Vec<BatchBufferContext<BatchBuffer<MockVertexBuffer>>> {
            &self.vertex_buffers
        }

        fn vertex_size(&self, object: &Shared<Polygon>) -> usize {
            object.borrow().vertex_size()
        }

        fn open(&mut self) {
            open_buffers(
                &self.index_buffer,
                &mut self.index_mapping,
                &self.vertex_buffers,
                &mut self.vertex_mappings,
            );
        }

        fn close(&mut self) {
            close_buffers(
                &self.index_buffer,
                &mut self.index_mapping,
                &self.vertex_buffers,
                &mut self.vertex_mappings,
            );
        }
    }
}
