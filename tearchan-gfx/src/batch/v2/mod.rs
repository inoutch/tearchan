use crate::batch::types::{BatchAttributeIndex, BatchTypeArray, BatchTypeTransform};
use crate::batch::v2::buffer::BatchBufferPointer;
use crate::batch::v2::object::BatchObject;
use crate::batch::v2::object_manager::{BatchObjectEvent, BatchObjectId, BatchObjectManager};
use crate::batch::v2::provider::BatchProvider;

pub mod batch2d;
pub mod batch3d;
pub mod batch_billboard;
pub mod batch_line;
pub mod buffer;
pub mod context;
pub mod object;
pub mod object_manager;
pub mod provider;

pub enum BatchEvent<'a> {
    Add {
        id: BatchObjectId,
    },
    Remove {
        id: BatchObjectId,
    },
    WriteToIndexBuffer {
        id: BatchObjectId,
        pointer: BatchBufferPointer,
        object: &'a BatchObject,
    },
    WriteToVertexBuffer {
        id: BatchObjectId,
        pointer: BatchBufferPointer,
        attribute: BatchAttributeIndex,
        object: &'a BatchObject,
    },
    ClearToIndexBuffer {
        pointer: BatchBufferPointer,
    },
    ClearToVertexBuffer {
        pointer: BatchBufferPointer,
    },
    ResizeIndexBuffer {
        len: usize,
    },
    ResizeVertexBuffer {
        len: usize,
    },
    Error,
}

pub struct Batch<T> {
    provider: T,
    manager: BatchObjectManager,
}

impl<T> Batch<T> {
    pub fn new(provider: T, index_len: usize, vertex_len: usize) -> Self {
        Batch {
            provider,
            manager: BatchObjectManager::new(index_len, vertex_len),
        }
    }

    pub fn provider(&self) -> &T {
        &self.provider
    }

    pub fn provider_mut(&mut self) -> &mut T {
        &mut self.provider
    }

    pub fn add(
        &mut self,
        indices: BatchTypeArray,
        vertices: Vec<BatchTypeArray>,
        order: Option<i32>,
    ) -> BatchObjectId {
        self.manager.add(indices, vertices, order)
    }

    pub fn remove(&mut self, id: BatchObjectId) {
        self.manager.remove(id);
    }

    pub fn replace_indices(&mut self, id: BatchObjectId, indices: BatchTypeArray) {
        self.manager.replace_indices(id, indices);
    }
    pub fn replace_vertices(&mut self, id: BatchObjectId, vertices: Vec<BatchTypeArray>) {
        self.manager.replace_vertices(id, vertices);
    }

    pub fn index_count(&self) -> usize {
        self.manager.index_allocator_len()
    }

    pub fn transform(
        &mut self,
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        transform: BatchTypeTransform,
    ) {
        self.manager.transform(id, attribute, transform);
    }
}

impl<TProvider> Batch<TProvider> {
    pub fn flush<'a>(&mut self, mut context: TProvider::Context)
    where
        TProvider: BatchProvider<'a>,
    {
        let provider = &mut self.provider;
        let manager = &mut self.manager;
        while let Some(event) = manager.pop_event() {
            match event {
                BatchObjectEvent::Add { id } => {
                    provider.run(&mut context, BatchEvent::Add { id });
                }
                BatchObjectEvent::Remove { id } => {
                    provider.run(&mut context, BatchEvent::Remove { id });
                }
                BatchObjectEvent::WriteToIndexBuffer { id } => {
                    let object = manager.get(id).unwrap();
                    provider.run(
                        &mut context,
                        BatchEvent::WriteToIndexBuffer {
                            id,
                            pointer: object.index_pointer(),
                            object,
                        },
                    );
                }
                BatchObjectEvent::WriteToVertexBuffer { id, attribute } => {
                    let object = manager.get(id).unwrap();
                    provider.run(
                        &mut context,
                        BatchEvent::WriteToVertexBuffer {
                            id,
                            pointer: object.vertex_pointer(),
                            attribute,
                            object,
                        },
                    );
                }
                BatchObjectEvent::ClearToIndexBuffer { pointer } => {
                    provider.run(&mut context, BatchEvent::ClearToIndexBuffer { pointer });
                }
                BatchObjectEvent::ClearToVertexBuffer { pointer } => {
                    provider.run(&mut context, BatchEvent::ClearToVertexBuffer { pointer });
                }
                BatchObjectEvent::ResizeIndexBuffer { len } => {
                    provider.run(&mut context, BatchEvent::ResizeIndexBuffer { len });
                }
                BatchObjectEvent::ResizeVertexBuffer { len } => {
                    provider.run(&mut context, BatchEvent::ResizeVertexBuffer { len });
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::batch::types::BatchTypeArray;
    use crate::batch::v2::buffer::BatchBuffer;
    use crate::batch::v2::provider::BatchProvider;
    use crate::batch::v2::{Batch, BatchEvent};
    use crate::v2::buffer::test::{TestBuffer, TestCopier, TestResizer, TestWriter};
    use nalgebra_glm::{vec2, vec3, Vec2, Vec3};
    use std::ops::{Deref, DerefMut};

    #[allow(dead_code)]
    struct TestBatchContext<'a> {
        resizer: &'a mut TestResizer,
        writer: &'a mut TestWriter,
        copier: &'a mut TestCopier,
    }

    struct TestBatch(Batch<TestBatchProvider>);

    impl Deref for TestBatch {
        type Target = Batch<TestBatchProvider>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for TestBatch {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl TestBatch {
        pub fn new(index_len: usize, vertex_len: usize) -> TestBatch {
            TestBatch(Batch::new(
                TestBatchProvider {
                    index_buffer: BatchBuffer::new(TestBuffer::new(vec![0; index_len])),
                    position_buffer: BatchBuffer::new(TestBuffer::new(
                        vec![(); vertex_len]
                            .iter()
                            .map(|_| vec3(0.0f32, 0.0f32, 0.0f32))
                            .collect(),
                    )),
                    texcoord_buffer: BatchBuffer::new(TestBuffer::new(
                        vec![(); vertex_len]
                            .iter()
                            .map(|_| vec2(0.0f32, 0.0f32))
                            .collect(),
                    )),
                },
                index_len,
                vertex_len,
            ))
        }
    }

    struct TestBatchProvider {
        index_buffer: BatchBuffer<TestBuffer<u32>, u32>,
        position_buffer: BatchBuffer<TestBuffer<Vec3>, Vec3>,
        texcoord_buffer: BatchBuffer<TestBuffer<Vec2>, Vec2>,
    }

    impl<'a> BatchProvider<'a> for TestBatchProvider {
        type Context = (&'a mut TestWriter, &'a mut TestCopier, &'a mut TestResizer);

        fn run(&mut self, context: &mut Self::Context, event: BatchEvent) {
            match event {
                BatchEvent::WriteToIndexBuffer {
                    pointer, object, ..
                } => {
                    self.index_buffer.write(
                        context.0,
                        pointer,
                        &object.get_v1u32_indices().unwrap(),
                    );
                }
                BatchEvent::WriteToVertexBuffer {
                    pointer,
                    attribute,
                    object,
                    ..
                } => match attribute {
                    0 => {
                        self.position_buffer.write(
                            context.0,
                            pointer,
                            &object.get_v3f32_vertices(attribute).unwrap(),
                        );
                    }
                    1 => self.texcoord_buffer.write(
                        context.0,
                        pointer,
                        &object.get_v2f32_vertices(attribute).unwrap(),
                    ),
                    _ => {}
                },
                BatchEvent::ClearToIndexBuffer { pointer } => {
                    self.index_buffer.clear(context.0, pointer);
                }
                BatchEvent::ClearToVertexBuffer { pointer } => {
                    self.position_buffer.clear(context.0, pointer);
                }
                BatchEvent::ResizeIndexBuffer { len } => {
                    self.index_buffer.resize(context.2, len);
                }
                BatchEvent::ResizeVertexBuffer { len } => {
                    self.position_buffer.resize(context.2, len);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_add() {
        let mut resizer = TestResizer;
        let mut writer = TestWriter;
        let mut copier = TestCopier;

        let mut batch = TestBatch::new(10, 10);
        batch.add(
            BatchTypeArray::V1U32 {
                data: vec![0, 1, 2],
            },
            vec![BatchTypeArray::V3F32 {
                data: vec![
                    vec3(0.0f32, 0.0f32, 0.0f32),
                    vec3(1.0f32, 0.0f32, 0.0f32),
                    vec3(1.0f32, 1.0f32, 0.0f32),
                ],
            }],
            None,
        );

        batch.flush((&mut writer, &mut copier, &mut resizer));

        assert_eq!(
            batch.provider().index_buffer.buffer().data.borrow()[0..batch.index_count()],
            vec![0, 1, 2]
        );
        assert_eq!(
            batch.provider().position_buffer.buffer().data.borrow()[0..batch.index_count()],
            vec![
                vec3(0.0f32, 0.0f32, 0.0f32),
                vec3(1.0f32, 0.0f32, 0.0f32),
                vec3(1.0f32, 1.0f32, 0.0f32)
            ]
        );
    }
}
