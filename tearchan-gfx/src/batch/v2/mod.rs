use crate::batch::types::{BatchAttributeIndex, BatchTypeArray, BatchTypeTransform};
use crate::batch::v2::buffer::BatchBufferPointer;
use crate::batch::v2::object::BatchObject;
use crate::batch::v2::object_manager::{BatchObjectEvent, BatchObjectId, BatchObjectManager};
use crate::batch::v2::provider::BatchProvider;

pub mod batch2d;
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
    Write {
        id: BatchObjectId,
        pointer: BatchBufferPointer,
        attribute: BatchAttributeIndex,
        object: &'a BatchObject,
    },
    Clear {
        pointer: BatchBufferPointer,
    },
    Resize {
        len: usize,
    },
    Error,
}

pub struct Batch<T> {
    provider: T,
    manager: BatchObjectManager,
}

impl<T> Batch<T> {
    pub fn new(provider: T, len: usize) -> Self {
        Batch {
            provider,
            manager: BatchObjectManager::new(len),
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
        data: Vec<BatchTypeArray>,
        len: usize,
        order: Option<i32>,
    ) -> BatchObjectId {
        self.manager.add(data, len, order)
    }

    pub fn remove(&mut self, id: BatchObjectId) {
        self.manager.remove(id);
    }

    pub fn replace(&mut self, id: BatchObjectId, len: usize, data: Vec<BatchTypeArray>) {
        self.manager.replace(id, len, data);
    }

    pub fn len(&self) -> usize {
        self.manager.allocator_len()
    }

    pub fn is_empty(&self) -> bool {
        self.manager.allocator_is_empty()
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
                BatchObjectEvent::Write { id, attribute } => {
                    let object = manager.get(id).unwrap();
                    provider.run(
                        &mut context,
                        BatchEvent::Write {
                            id,
                            pointer: object.pointer(),
                            attribute,
                            object,
                        },
                    );
                }
                BatchObjectEvent::Clear { pointer } => {
                    provider.run(&mut context, BatchEvent::Clear { pointer });
                }
                BatchObjectEvent::Resize { len } => {
                    provider.run(&mut context, BatchEvent::Resize { len });
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::batch::types::BatchTypeArray;
    use crate::batch::v2::buffer::{IndexBatchBuffer, VertexBatchBuffer};
    use crate::batch::v2::provider::BatchProvider;
    use crate::batch::v2::{Batch, BatchEvent};
    use crate::v2::buffer::test::{TestBuffer, TestCopier, TestResizer, TestWriter};
    use nalgebra_glm::{vec3, Vec3};
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
        pub fn new(len: usize) -> TestBatch {
            TestBatch(Batch::new(
                TestBatchProvider {
                    index_buffer: IndexBatchBuffer::new(TestBuffer::new(vec![0; len])),
                    position_buffer: VertexBatchBuffer::new(TestBuffer::new(vec![])),
                },
                len,
            ))
        }
    }

    struct TestBatchProvider {
        index_buffer: IndexBatchBuffer<TestBuffer<u32>>,
        position_buffer: VertexBatchBuffer<TestBuffer<Vec3>, Vec3>,
    }

    impl<'a> BatchProvider<'a> for TestBatchProvider {
        type Context = (&'a mut TestWriter, &'a mut TestCopier, &'a mut TestResizer);

        fn run(&mut self, context: &mut Self::Context, event: BatchEvent) {
            match event {
                BatchEvent::Write {
                    pointer,
                    attribute,
                    object,
                    ..
                } => match attribute {
                    0 => {
                        self.index_buffer.write(
                            context.0,
                            pointer,
                            &object.get_v1u32_data(attribute).unwrap(),
                        );
                    }
                    1 => {
                        self.position_buffer.write(
                            context.0,
                            pointer,
                            &object.get_v3f32_data(attribute).unwrap(),
                        );
                    }
                    _ => {}
                },
                BatchEvent::Clear { pointer } => {
                    self.index_buffer.clear(context.0, pointer);
                    self.position_buffer.clear(context.0, pointer);
                }
                BatchEvent::Resize { len } => {
                    self.index_buffer.resize(context.2, len);
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

        let mut batch = TestBatch::new(10);
        batch.add(
            vec![
                BatchTypeArray::V1U32 {
                    data: vec![0, 1, 2],
                },
                BatchTypeArray::V3F32 {
                    data: vec![
                        vec3(0.0f32, 0.0f32, 0.0f32),
                        vec3(1.0f32, 0.0f32, 0.0f32),
                        vec3(1.0f32, 1.0f32, 0.0f32),
                    ],
                },
            ],
            3,
            None,
        );

        batch.flush((&mut writer, &mut copier, &mut resizer));

        assert_eq!(
            batch.provider().index_buffer.buffer().data.borrow()[0..batch.len()],
            vec![0, 1, 2]
        );
        assert_eq!(
            batch.provider().position_buffer.buffer().data.borrow()[0..batch.len()],
            vec![
                vec3(0.0f32, 0.0f32, 0.0f32),
                vec3(1.0f32, 0.0f32, 0.0f32),
                vec3(1.0f32, 1.0f32, 0.0f32)
            ]
        );
    }
}
