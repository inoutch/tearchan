use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::BatchContext;
use crate::core::graphic::hal::buffer_interface::BufferInterface;
use crate::extension::shared::Shared;
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

pub trait BatchProvider<TObject, TIndexBuffer: BufferInterface, TBuffer: BufferInterface> {
    fn update(&mut self, context: &Rc<BatchContext<TObject>>);

    fn index_buffer(&self) -> &BatchBuffer<TIndexBuffer>;

    fn index_buffer_mut(&mut self) -> &mut BatchBuffer<TIndexBuffer>;

    fn index_size(&self, object: &Shared<TObject>) -> usize;

    fn contexts_mut(&mut self) -> &mut Vec<BatchBufferContext<BatchBuffer<TBuffer>>>;

    fn contexts(&self) -> &Vec<BatchBufferContext<BatchBuffer<TBuffer>>>;

    fn vertex_size(&self, object: &Shared<TObject>) -> usize;

    fn open(&mut self);

    fn close(&mut self);
}

#[cfg(test)]
pub mod test {
    use crate::core::graphic::batch::batch_buffer::BatchBuffer;
    use crate::core::graphic::batch::batch_provider::{BatchBufferContext, BatchProvider};
    use crate::core::graphic::batch::BatchContext;
    use crate::core::graphic::hal::buffer_interface::BufferInterface;
    use crate::core::graphic::hal::index_buffer::test::{
        MockIndexBuffer, MockIndexBufferMappedMemory,
    };
    use crate::core::graphic::hal::vertex_buffer::test::{
        MockVertexBuffer, MockVertexBufferMappedMemory,
    };
    use crate::core::graphic::polygon::Polygon;
    use crate::extension::shared::Shared;
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
        fn update(&mut self, context: &Rc<BatchContext<Polygon>>) {
            // update positions, colors, texcoords, normals, indices
            let mut object = context.object.borrow_mut();
            let index_mapping = match &mut self.index_mapping {
                Some(mapping) => mapping,
                None => return,
            };
            object.copy_indices_into(index_mapping, context.index_pointer.borrow().first);
            object.copy_positions_into(
                &mut self.vertex_mappings[0],
                context.vertex_pointers[0].borrow().first,
            );
            object.copy_colors_into(
                &mut self.vertex_mappings[1],
                context.vertex_pointers[1].borrow().first,
            );
            object.copy_texcoords_into(
                &mut self.vertex_mappings[2],
                context.vertex_pointers[2].borrow().first,
            );
            object.copy_normals_into(
                &mut self.vertex_mappings[3],
                context.vertex_pointers[3].borrow().first,
            );
        }

        fn index_size(&self, object: &Shared<Polygon>) -> usize {
            object.borrow().index_size()
        }

        fn index_buffer(&self) -> &BatchBuffer<MockIndexBuffer> {
            &self.index_buffer
        }

        fn index_buffer_mut(&mut self) -> &mut BatchBuffer<MockIndexBuffer> {
            &mut self.index_buffer
        }

        fn contexts_mut(&mut self) -> &mut Vec<BatchBufferContext<BatchBuffer<MockVertexBuffer>>> {
            &mut self.vertex_buffers
        }

        fn contexts(&self) -> &Vec<BatchBufferContext<BatchBuffer<MockVertexBuffer>>> {
            &self.vertex_buffers
        }

        fn vertex_size(&self, object: &Shared<Polygon>) -> usize {
            object.borrow().vertex_size()
        }

        fn open(&mut self) {
            self.index_mapping = Some(self.index_buffer.buffer().open(0, self.index_buffer.size()));
            self.vertex_mappings = self
                .vertex_buffers
                .iter()
                .map(|context| context.buffer.buffer().open(0, context.buffer.size()))
                .collect();
        }

        fn close(&mut self) {
            let index_mapping = std::mem::replace(&mut self.index_mapping, None).unwrap();
            self.index_buffer.buffer().close(index_mapping);

            let mut index = 0;
            while !self.vertex_mappings.is_empty() {
                let vertex_mapping = self.vertex_mappings.remove(0);
                self.vertex_buffers[index]
                    .buffer
                    .buffer()
                    .close(vertex_mapping);
                index += 1;
            }
        }
    }
}
