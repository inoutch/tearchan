use crate::batch::buffer::{BatchBufferAllocator, BatchBufferAllocatorEvent, BatchBufferPointer};
use crate::batch::object::BatchObject;
use crate::batch::types::{BatchAttributeIndex, BatchTypeArray, BatchTypeTransform};
use std::collections::{HashMap, HashSet, VecDeque};
use tearchan_util::id_manager::IdManager;

const DEFAULT_ORDER: i32 = i32::MAX;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct BatchObjectId(u64);

impl BatchObjectId {
    fn next(&self) -> Self {
        BatchObjectId(self.0 + 1)
    }
}

#[derive(Debug)]
pub enum BatchObjectEvent {
    Add {
        id: BatchObjectId,
    },
    Remove {
        id: BatchObjectId,
    },
    WriteToIndexBuffer {
        id: BatchObjectId,
    },
    WriteToVertexBuffer {
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
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
}

#[derive(Hash, Eq, PartialEq)]
enum BatchObjectKey {
    Index(BatchObjectId),
    Vertex((BatchObjectId, BatchAttributeIndex)),
}

pub struct BatchObjectManager {
    id_manager: IdManager<BatchObjectId>,
    objects: HashMap<BatchObjectId, BatchObject>,
    object_ids_grouped_by_index_pointer: HashMap<BatchBufferPointer, BatchObjectId>,
    object_ids_grouped_by_vertex_pointer: HashMap<BatchBufferPointer, BatchObjectId>,
    objects_will_be_rewritten: HashSet<BatchObjectKey>,
    index_allocator: BatchBufferAllocator,
    vertex_allocator: BatchBufferAllocator,
    events: VecDeque<BatchObjectEvent>,
    index_len: usize,
    vertex_len: usize,
    should_sort_indices: bool,
}

impl BatchObjectManager {
    pub fn new(index_len: usize, vertex_len: usize) -> Self {
        BatchObjectManager {
            id_manager: IdManager::new(BatchObjectId(0), |id| id.next()),
            objects: HashMap::new(),
            object_ids_grouped_by_index_pointer: HashMap::new(),
            object_ids_grouped_by_vertex_pointer: HashMap::new(),
            objects_will_be_rewritten: HashSet::new(),
            index_allocator: BatchBufferAllocator::default(),
            vertex_allocator: BatchBufferAllocator::default(),
            events: VecDeque::new(),
            index_len,
            vertex_len,
            should_sort_indices: false,
        }
    }

    pub fn pop_event(&mut self) -> Option<BatchObjectEvent> {
        if self.should_sort_indices {
            let object_ids_grouped_by_index_pointer = &self.object_ids_grouped_by_index_pointer;
            let objects = &self.objects;
            self.index_allocator.sort_by(|a, b| {
                let a = object_ids_grouped_by_index_pointer.get(a).unwrap();
                let b = object_ids_grouped_by_index_pointer.get(b).unwrap();
                let a = objects.get(a).unwrap();
                let b = objects.get(b).unwrap();
                a.order().cmp(&b.order())
            });
            self.should_sort_indices = false;
        }
        while let Some(event) = self.index_allocator.pop_event() {
            match event {
                BatchBufferAllocatorEvent::Write(pointer) => {
                    let object_id = self
                        .object_ids_grouped_by_index_pointer
                        .get(&pointer)
                        .unwrap();
                    self.events
                        .push_back(BatchObjectEvent::WriteToIndexBuffer { id: *object_id });
                }
                BatchBufferAllocatorEvent::Clear(pointer) => {
                    self.events
                        .push_back(BatchObjectEvent::ClearToIndexBuffer { pointer });
                }
                BatchBufferAllocatorEvent::ReallocateAll { pairs } => {
                    let mut object_ids_grouped_by_index_pointer = HashMap::new();
                    for pair in pairs {
                        let object_id = *self
                            .object_ids_grouped_by_index_pointer
                            .get(&pair.from)
                            .unwrap();
                        let object = self.objects.get_mut(&object_id).unwrap();
                        object.set_index_pointer(pair.to);
                        object_ids_grouped_by_index_pointer.insert(pair.to, object_id);
                        self.objects_will_be_rewritten
                            .insert(BatchObjectKey::Index(object_id));
                    }
                    self.object_ids_grouped_by_index_pointer = object_ids_grouped_by_index_pointer;
                }
            }
        }

        while let Some(event) = self.vertex_allocator.pop_event() {
            match event {
                BatchBufferAllocatorEvent::Write(pointer) => {
                    let object_id = self
                        .object_ids_grouped_by_vertex_pointer
                        .get(&pointer)
                        .unwrap();
                    let object = self.objects.get(object_id).unwrap();
                    for (i, _) in object.vertices().iter().enumerate() {
                        self.events
                            .push_back(BatchObjectEvent::WriteToVertexBuffer {
                                id: *object_id,
                                attribute: i as BatchAttributeIndex,
                            });
                    }
                }
                BatchBufferAllocatorEvent::Clear(pointer) => {
                    self.events
                        .push_back(BatchObjectEvent::ClearToVertexBuffer { pointer });
                }
                BatchBufferAllocatorEvent::ReallocateAll { pairs } => {
                    let mut object_ids_grouped_by_vertex_pointer = HashMap::new();
                    for pair in pairs {
                        let object_id = *self
                            .object_ids_grouped_by_vertex_pointer
                            .get(&pair.from)
                            .unwrap();
                        let object = self.objects.get_mut(&object_id).unwrap();

                        object.set_vertex_pointer(pair.to);
                        object_ids_grouped_by_vertex_pointer.insert(pair.to, object_id);

                        for i in 0u32..object.vertices().len() as u32 {
                            self.objects_will_be_rewritten
                                .insert(BatchObjectKey::Vertex((object_id, i)));
                        }
                    }
                    self.object_ids_grouped_by_vertex_pointer =
                        object_ids_grouped_by_vertex_pointer;
                }
            }
        }

        let event = self.events.pop_front()?;
        match &event {
            BatchObjectEvent::WriteToIndexBuffer { id } => {
                self.objects_will_be_rewritten
                    .remove(&BatchObjectKey::Index(*id));
            }
            BatchObjectEvent::WriteToVertexBuffer { id, attribute } => {
                self.objects_will_be_rewritten
                    .remove(&BatchObjectKey::Vertex((*id, *attribute)));
            }
            _ => {}
        }
        Some(event)
    }

    pub fn clear_events(&mut self) {
        self.objects_will_be_rewritten.clear();
    }

    #[inline]
    pub fn get(&self, id: BatchObjectId) -> Option<&BatchObject> {
        self.objects.get(&id)
    }

    pub fn add(
        &mut self,
        indices: BatchTypeArray,
        vertices: Vec<BatchTypeArray>,
        order: Option<i32>,
    ) -> BatchObjectId {
        if order.is_some() {
            self.should_sort_indices = true;
        }
        let mut iter = vertices.iter();
        let vertex_len = iter.next().map(|array| array.len()).unwrap_or(0);
        for array in iter {
            assert_eq!(array.len(), vertex_len);
        }

        let id = self.id_manager.gen();
        let order = order.unwrap_or(DEFAULT_ORDER);
        let transforms = vec![BatchTypeTransform::None; vertices.len()];

        let index_pointer = self.index_allocator.allocate(indices.len());
        if self.index_allocator.len() > self.index_len {
            self.index_len = self.index_allocator.len() * 2;
            self.events.push_back(BatchObjectEvent::ResizeIndexBuffer {
                len: self.index_len,
            });
        }
        let vertex_pointer = self.vertex_allocator.allocate(vertex_len);
        if self.vertex_allocator.len() > self.vertex_len {
            self.vertex_len = self.vertex_allocator.len() * 2;
            self.events.push_back(BatchObjectEvent::ResizeVertexBuffer {
                len: self.vertex_len,
            });
        }

        let object = BatchObject::new(
            index_pointer,
            vertex_pointer,
            indices,
            vertices,
            transforms,
            order,
        );

        self.objects_will_be_rewritten
            .insert(BatchObjectKey::Index(id));
        for i in 0u32..object.vertices().len() as u32 {
            self.objects_will_be_rewritten
                .insert(BatchObjectKey::Vertex((id, i)));
        }
        self.events.push_back(BatchObjectEvent::Add { id });
        self.objects.insert(id, object);
        self.object_ids_grouped_by_index_pointer
            .insert(index_pointer, id);
        self.object_ids_grouped_by_vertex_pointer
            .insert(vertex_pointer, id);
        id
    }

    pub fn remove(&mut self, id: BatchObjectId) -> Option<BatchObject> {
        let object = self.objects.remove(&id)?;
        self.events.push_back(BatchObjectEvent::Remove { id });
        self.index_allocator.free(object.index_pointer());
        self.vertex_allocator.free(object.vertex_pointer());
        self.object_ids_grouped_by_index_pointer
            .remove(&object.index_pointer());
        self.object_ids_grouped_by_vertex_pointer
            .remove(&object.vertex_pointer());
        self.objects_will_be_rewritten
            .remove(&BatchObjectKey::Index(id));
        for attribute in 0..object.vertices().len() {
            self.objects_will_be_rewritten
                .remove(&BatchObjectKey::Vertex((id, attribute as u32)));
        }
        Some(object)
    }

    pub fn transform(
        &mut self,
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        transform: BatchTypeTransform,
    ) {
        let object = match self.objects.get_mut(&id) {
            Some(object) => object,
            None => return,
        };
        object.set_transform(attribute, transform);

        let key = BatchObjectKey::Vertex((id, attribute));
        if self.objects_will_be_rewritten.get(&key).is_none() {
            self.events
                .push_back(BatchObjectEvent::WriteToVertexBuffer { id, attribute });
            self.objects_will_be_rewritten.insert(key);
        }
    }

    pub fn rewrite_indices(&mut self, id: BatchObjectId, indices: BatchTypeArray) {
        let object = match self.objects.get_mut(&id) {
            Some(object) => object,
            None => return,
        };
        assert_eq!(indices.len(), object.indices().len());
        object.set_indices(indices);
    }

    pub fn rewrite_vertices(
        &mut self,
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        vertices: BatchTypeArray,
    ) {
        let object = match self.objects.get_mut(&id) {
            Some(object) => object,
            None => return,
        };
        let prev_data = match object.vertices().get(attribute as usize) {
            Some(data) => data,
            None => return,
        };
        assert_eq!(vertices.len(), prev_data.len());

        object.set_vertices(attribute, vertices);
        let key = BatchObjectKey::Vertex((id, attribute));
        if self.objects_will_be_rewritten.get(&key).is_none() {
            self.events
                .push_back(BatchObjectEvent::WriteToVertexBuffer { id, attribute });
            self.objects_will_be_rewritten.insert(key);
        }
    }

    pub fn replace_indices(&mut self, id: BatchObjectId, indices: BatchTypeArray) {
        let object = match self.objects.get_mut(&id) {
            Some(object) => object,
            None => return,
        };
        if object.indices().len() == indices.len() {
            self.rewrite_indices(id, indices);
            return;
        }

        self.object_ids_grouped_by_index_pointer
            .remove(&object.index_pointer());

        let new_pointer = self
            .index_allocator
            .reallocate(object.index_pointer(), indices.len());
        object.set_index_pointer(new_pointer);
        object.set_indices(indices);
        self.object_ids_grouped_by_index_pointer
            .insert(new_pointer, id);

        self.objects_will_be_rewritten
            .insert(BatchObjectKey::Index(id));
    }

    pub fn replace_vertices(&mut self, id: BatchObjectId, vertices: Vec<BatchTypeArray>) {
        let object = match self.objects.get_mut(&id) {
            Some(object) => object,
            None => return,
        };

        let mut iter = vertices.iter();
        let vertex_len = iter.next().map(|array| array.len()).unwrap_or(0);
        for array in iter {
            assert_eq!(array.len(), vertex_len);
        }
        if object
            .vertices()
            .first()
            .map(|vertices| vertices.len() == vertex_len)
            .unwrap_or(false)
        {
            for (i, data) in vertices.into_iter().enumerate() {
                self.rewrite_vertices(id, i as u32, data);
            }
            return;
        }

        self.object_ids_grouped_by_vertex_pointer
            .remove(&object.vertex_pointer());

        let new_pointer = self
            .vertex_allocator
            .reallocate(object.vertex_pointer(), vertex_len);
        object.set_vertex_pointer(new_pointer);
        for (attribute, vertices) in vertices.into_iter().enumerate() {
            object.set_vertices(attribute as u32, vertices);
        }
        self.object_ids_grouped_by_vertex_pointer
            .insert(new_pointer, id);

        for i in 0u32..object.vertices().len() as u32 {
            self.objects_will_be_rewritten
                .insert(BatchObjectKey::Vertex((id, i)));
        }
    }

    pub fn index_allocator_len(&self) -> usize {
        self.index_allocator.len()
    }
}

#[cfg(test)]
mod test {
    use crate::batch::object_manager::{BatchObjectEvent, BatchObjectManager};
    use crate::batch::types::{BatchTypeArray, BatchTypeTransform};
    use nalgebra_glm::{vec2, vec3, Mat2, Mat3};

    fn convert_events(manager: &mut BatchObjectManager) -> Vec<BatchObjectEvent> {
        let mut events = Vec::new();
        while let Some(event) = manager.pop_event() {
            events.push(event);
        }
        events
    }

    #[test]
    fn test_valid_len() {
        let mut manager = BatchObjectManager::new(100, 100);
        manager.add(
            BatchTypeArray::V1U32 { data: vec![0] },
            vec![
                BatchTypeArray::V1F32 { data: vec![0.0f32] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(0.0f32, 0.0f32)],
                },
            ],
            None,
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_len() {
        let mut manager = BatchObjectManager::new(100, 100);
        manager.add(
            BatchTypeArray::V1U32 { data: vec![0, 1] },
            vec![
                BatchTypeArray::V1F32 {
                    data: vec![0.0f32, 0.0f32],
                },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(0.0f32, 0.0f32)],
                },
            ],
            None,
        );
    }

    #[test]
    fn test_cleanup() {
        let mut manager = BatchObjectManager::new(100, 100);
        let indices = BatchTypeArray::V1U32 { data: vec![0] };
        let vertices = vec![
            BatchTypeArray::V1F32 { data: vec![0.0f32] },
            BatchTypeArray::V2F32 {
                data: vec![vec2(0.0f32, 0.0f32)],
            },
        ];
        let id0 = manager.add(indices, vertices, None);

        let indices = BatchTypeArray::V1U32 { data: vec![0] };
        let vertices = vec![
            BatchTypeArray::V1F32 { data: vec![0.0f32] },
            BatchTypeArray::V2F32 {
                data: vec![vec2(0.0f32, 0.0f32)],
            },
        ];
        let id1 = manager.add(indices, vertices, None);
        manager.remove(id0);
        manager.remove(id1);

        assert_eq!(manager.objects.len(), 0);
        assert_eq!(manager.object_ids_grouped_by_index_pointer.len(), 0);
        assert_eq!(manager.object_ids_grouped_by_vertex_pointer.len(), 0);
        assert_eq!(manager.objects_will_be_rewritten.len(), 0);
        assert_eq!(manager.index_allocator.len(), 0);
        assert_eq!(manager.vertex_allocator.len(), 0);
    }

    #[test]
    fn test_events() {
        let mut manager = BatchObjectManager::new(100, 100);
        let id0 = manager.add(
            BatchTypeArray::V1U32 { data: vec![0] },
            vec![
                BatchTypeArray::V1F32 { data: vec![1.0f32] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(1.0f32, 1.0f32)],
                },
            ],
            None,
        );
        let id1 = manager.add(
            BatchTypeArray::V1U32 { data: vec![0] },
            vec![
                BatchTypeArray::V1F32 { data: vec![2.0f32] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(2.0f32, 2.0f32)],
                },
            ],
            None,
        );

        insta::assert_debug_snapshot!(convert_events(&mut manager));

        manager.transform(
            id0,
            1,
            BatchTypeTransform::Mat3F32 {
                m: Mat3::new(
                    1.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32,
                ),
            },
        );
        manager.transform(
            id1,
            0,
            BatchTypeTransform::Mat2F32 {
                m: Mat2::new(1.0f32, 0.0f32, 0.0f32, 1.0f32),
            },
        );

        insta::assert_debug_snapshot!(convert_events(&mut manager));

        manager.remove(id0);

        insta::assert_debug_snapshot!(convert_events(&mut manager));

        manager.remove(id1);

        insta::assert_debug_snapshot!(convert_events(&mut manager));
    }

    #[test]
    fn test_resize_event() {
        let mut manager = BatchObjectManager::new(1, 2);
        let _id0 = manager.add(
            BatchTypeArray::V1U32 { data: vec![0] },
            vec![
                BatchTypeArray::V1U32 { data: vec![1, 1] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(1.0f32, 1.0f32), vec2(1.0f32, 1.0f32)],
                },
                BatchTypeArray::V3F32 {
                    data: vec![vec3(1.0f32, 1.0f32, 1.0f32), vec3(1.0f32, 1.0f32, 1.0f32)],
                },
            ],
            None,
        );

        insta::assert_debug_snapshot!(convert_events(&mut manager));

        let _id1 = manager.add(
            BatchTypeArray::V1U32 { data: vec![0] },
            vec![
                BatchTypeArray::V1U32 {
                    data: vec![2, 2, 2],
                },
                BatchTypeArray::V2F32 {
                    data: vec![
                        vec2(2.0f32, 2.0f32),
                        vec2(2.0f32, 2.0f32),
                        vec2(2.0f32, 2.0f32),
                    ],
                },
                BatchTypeArray::V3F32 {
                    data: vec![
                        vec3(2.0f32, 2.0f32, 2.0f32),
                        vec3(2.0f32, 2.0f32, 2.0f32),
                        vec3(2.0f32, 2.0f32, 2.0f32),
                    ],
                },
            ],
            None,
        );
        insta::assert_debug_snapshot!(convert_events(&mut manager));
    }

    #[test]
    fn test_sort_by_order() {
        let mut manager = BatchObjectManager::new(1, 2);
        let _id0 = manager.add(
            BatchTypeArray::V1U32 { data: vec![0] },
            vec![
                BatchTypeArray::V1U32 { data: vec![1, 1] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(1.0f32, 1.0f32), vec2(1.0f32, 1.0f32)],
                },
                BatchTypeArray::V3F32 {
                    data: vec![vec3(1.0f32, 1.0f32, 1.0f32), vec3(1.0f32, 1.0f32, 1.0f32)],
                },
            ],
            Some(6),
        );
        let _id1 = manager.add(
            BatchTypeArray::V1U32 { data: vec![1] },
            vec![
                BatchTypeArray::V1U32 { data: vec![1, 1] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(1.0f32, 1.0f32), vec2(1.0f32, 1.0f32)],
                },
                BatchTypeArray::V3F32 {
                    data: vec![vec3(1.0f32, 1.0f32, 1.0f32), vec3(1.0f32, 1.0f32, 1.0f32)],
                },
            ],
            Some(2),
        );
        let _id2 = manager.add(
            BatchTypeArray::V1U32 { data: vec![2] },
            vec![
                BatchTypeArray::V1U32 { data: vec![1, 1] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(1.0f32, 1.0f32), vec2(1.0f32, 1.0f32)],
                },
                BatchTypeArray::V3F32 {
                    data: vec![vec3(1.0f32, 1.0f32, 1.0f32), vec3(1.0f32, 1.0f32, 1.0f32)],
                },
            ],
            Some(4),
        );

        insta::assert_debug_snapshot!(convert_events(&mut manager));

        let _id3 = manager.add(
            BatchTypeArray::V1U32 { data: vec![3] },
            vec![
                BatchTypeArray::V1U32 { data: vec![1, 1] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(1.0f32, 1.0f32), vec2(1.0f32, 1.0f32)],
                },
                BatchTypeArray::V3F32 {
                    data: vec![vec3(1.0f32, 1.0f32, 1.0f32), vec3(1.0f32, 1.0f32, 1.0f32)],
                },
            ],
            Some(5),
        );
        let _id4 = manager.add(
            BatchTypeArray::V1U32 { data: vec![4] },
            vec![
                BatchTypeArray::V1U32 { data: vec![1, 1] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(1.0f32, 1.0f32), vec2(1.0f32, 1.0f32)],
                },
                BatchTypeArray::V3F32 {
                    data: vec![vec3(1.0f32, 1.0f32, 1.0f32), vec3(1.0f32, 1.0f32, 1.0f32)],
                },
            ],
            Some(3),
        );

        insta::assert_debug_snapshot!(convert_events(&mut manager));
    }
}
