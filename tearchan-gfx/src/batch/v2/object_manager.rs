use crate::batch::types::{BatchAttributeIndex, BatchTypeArray, BatchTypeTransform};
use crate::batch::v2::buffer::{
    BatchBufferAllocator, BatchBufferAllocatorEvent, BatchBufferPointer,
};
use crate::batch::v2::object::BatchObject;
use crate::batch::DEFAULT_ORDER;
use std::collections::{HashMap, HashSet, VecDeque};
use tearchan_util::id_manager::IdManager;

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
    Write {
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
    },
    Clear {
        pointer: BatchBufferPointer,
    },
    Resize {
        len: usize,
    },
}

pub struct BatchObjectManager {
    id_manager: IdManager<BatchObjectId>,
    objects: HashMap<BatchObjectId, BatchObject>,
    object_ids_grouped_by_pointer: HashMap<BatchBufferPointer, BatchObjectId>,
    objects_will_be_rewritten: HashSet<(BatchObjectId, BatchAttributeIndex)>,
    allocator: BatchBufferAllocator,
    events: VecDeque<BatchObjectEvent>,
    len: usize,
}

impl BatchObjectManager {
    pub fn new(len: usize) -> Self {
        BatchObjectManager {
            id_manager: IdManager::new(BatchObjectId(0), |id| id.next()),
            objects: HashMap::new(),
            objects_will_be_rewritten: HashSet::new(),
            object_ids_grouped_by_pointer: HashMap::new(),
            allocator: BatchBufferAllocator::default(),
            events: VecDeque::new(),
            len,
        }
    }

    pub fn pop_event(&mut self) -> Option<BatchObjectEvent> {
        while let Some(event) = self.allocator.pop_event() {
            match event {
                BatchBufferAllocatorEvent::Write(pointer) => {
                    let object_id = self.object_ids_grouped_by_pointer.get(&pointer).unwrap();
                    let object = self.objects.get(object_id).unwrap();
                    for (i, _) in object.data().iter().enumerate() {
                        self.events.push_back(BatchObjectEvent::Write {
                            id: *object_id,
                            attribute: i as BatchAttributeIndex,
                        });
                    }
                }
                BatchBufferAllocatorEvent::Clear(pointer) => {
                    self.events.push_back(BatchObjectEvent::Clear { pointer });
                }
                BatchBufferAllocatorEvent::Reallocate { from, to } => {
                    let object_id = *self.object_ids_grouped_by_pointer.get(&from).unwrap();
                    let object = self.objects.get_mut(&object_id).unwrap();
                    self.object_ids_grouped_by_pointer.remove(&object.pointer());

                    object.set_pointer(to);
                    self.object_ids_grouped_by_pointer.insert(to, object_id);

                    for i in 0u32..object.data().len() as u32 {
                        self.objects_will_be_rewritten.insert((object_id, i));
                    }
                }
            }
        }
        let event = self.events.pop_front()?;
        match &event {
            BatchObjectEvent::Add { .. } => {}
            BatchObjectEvent::Remove { .. } => {}
            BatchObjectEvent::Write { id, attribute } => {
                self.objects_will_be_rewritten.remove(&(*id, *attribute));
            }
            BatchObjectEvent::Clear { .. } => {}
            BatchObjectEvent::Resize { .. } => {}
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
        data: Vec<BatchTypeArray>,
        len: usize,
        order: Option<i32>,
    ) -> BatchObjectId {
        for datum in data.iter() {
            assert_eq!(datum.len(), len);
        }

        let id = self.id_manager.gen();
        let order = order.unwrap_or(DEFAULT_ORDER);
        let transforms = vec![BatchTypeTransform::None; data.len()];
        let pointer = self.allocator.allocate(len);
        let object = BatchObject::new(pointer, data, transforms, order);

        if self.allocator.len() > self.len {
            self.len = self.allocator.len() * 2;
            self.events
                .push_back(BatchObjectEvent::Resize { len: self.len });
        }

        for i in 0u32..object.data().len() as u32 {
            self.objects_will_be_rewritten.insert((id, i));
        }
        self.events.push_back(BatchObjectEvent::Add { id });
        self.objects.insert(id, object);
        self.object_ids_grouped_by_pointer.insert(pointer, id);
        id
    }

    pub fn remove(&mut self, id: BatchObjectId) -> Option<BatchObject> {
        self.events.push_back(BatchObjectEvent::Remove { id });
        let object = self.objects.remove(&id)?;
        self.allocator.free(object.pointer());
        self.object_ids_grouped_by_pointer.remove(&object.pointer());
        for attribute in 0..object.data().len() {
            self.objects_will_be_rewritten
                .remove(&(id, attribute as u32));
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

        let key = (id, attribute);
        if self.objects_will_be_rewritten.get(&key).is_none() {
            self.events
                .push_back(BatchObjectEvent::Write { id, attribute });
            self.objects_will_be_rewritten.insert(key);
        }
    }

    pub fn rewrite(
        &mut self,
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        data: BatchTypeArray,
    ) {
        let object = match self.objects.get_mut(&id) {
            Some(object) => object,
            None => return,
        };
        let prev_data = match object.data().get(attribute as usize) {
            Some(data) => data,
            None => return,
        };
        assert_eq!(data.len(), prev_data.len());

        object.set_data(attribute, data);
        let key = (id, attribute);
        if self.objects_will_be_rewritten.get(&key).is_none() {
            self.events
                .push_back(BatchObjectEvent::Write { id, attribute });
            self.objects_will_be_rewritten.insert(key);
        }
    }

    pub fn replace(&mut self, id: BatchObjectId, len: usize, data: Vec<BatchTypeArray>) {
        for datum in data.iter() {
            assert_eq!(datum.len(), len);
        }

        let object = match self.objects.get_mut(&id) {
            Some(object) => object,
            None => return,
        };
        if !object.data().is_empty() && object.data().get(0).unwrap().len() == len {
            for (i, data) in data.into_iter().enumerate() {
                self.rewrite(id, i as u32, data);
            }
            return;
        }

        self.object_ids_grouped_by_pointer.remove(&object.pointer());

        let new_pointer = self.allocator.reallocate(object.pointer(), len);
        object.set_pointer(new_pointer);
        self.object_ids_grouped_by_pointer.insert(new_pointer, id);

        for i in 0u32..object.data().len() as u32 {
            self.objects_will_be_rewritten.insert((id, i));
        }
    }

    pub fn allocator_len(&self) -> usize {
        self.allocator.len()
    }

    pub fn allocator_is_empty(&self) -> bool {
        self.allocator.is_empty()
    }
}

#[cfg(test)]
mod test {
    use crate::batch::types::{BatchTypeArray, BatchTypeTransform};
    use crate::batch::v2::object_manager::{BatchObjectEvent, BatchObjectManager};
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
        let mut manager = BatchObjectManager::new(100);
        manager.add(
            vec![
                BatchTypeArray::V1F32 { data: vec![0.0f32] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(0.0f32, 0.0f32)],
                },
            ],
            1,
            None,
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_len() {
        let mut manager = BatchObjectManager::new(100);
        manager.add(
            vec![
                BatchTypeArray::V1F32 {
                    data: vec![0.0f32, 0.0f32],
                },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(0.0f32, 0.0f32)],
                },
            ],
            2,
            None,
        );
    }

    #[test]
    fn test_cleanup() {
        let mut manager = BatchObjectManager::new(100);
        let data = vec![
            BatchTypeArray::V1F32 { data: vec![0.0f32] },
            BatchTypeArray::V2F32 {
                data: vec![vec2(0.0f32, 0.0f32)],
            },
        ];
        let id0 = manager.add(data, 1, None);

        let data = vec![
            BatchTypeArray::V1F32 { data: vec![0.0f32] },
            BatchTypeArray::V2F32 {
                data: vec![vec2(0.0f32, 0.0f32)],
            },
        ];
        let id1 = manager.add(data, 1, None);
        manager.remove(id0);
        manager.remove(id1);

        assert_eq!(manager.objects.len(), 0);
        assert_eq!(manager.object_ids_grouped_by_pointer.len(), 0);
        assert_eq!(manager.objects_will_be_rewritten.len(), 0);
        assert_eq!(manager.allocator.len(), 0);
    }

    #[test]
    fn test_events() {
        let mut manager = BatchObjectManager::new(100);
        let id0 = manager.add(
            vec![
                BatchTypeArray::V1F32 { data: vec![1.0f32] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(1.0f32, 1.0f32)],
                },
            ],
            1,
            None,
        );
        let id1 = manager.add(
            vec![
                BatchTypeArray::V1F32 { data: vec![2.0f32] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(2.0f32, 2.0f32)],
                },
            ],
            1,
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
        let mut manager = BatchObjectManager::new(2);
        let _id0 = manager.add(
            vec![
                BatchTypeArray::V1U32 { data: vec![1, 1] },
                BatchTypeArray::V2F32 {
                    data: vec![vec2(1.0f32, 1.0f32), vec2(1.0f32, 1.0f32)],
                },
                BatchTypeArray::V3F32 {
                    data: vec![vec3(1.0f32, 1.0f32, 1.0f32), vec3(1.0f32, 1.0f32, 1.0f32)],
                },
            ],
            2,
            None,
        );

        insta::assert_debug_snapshot!(convert_events(&mut manager));

        let _id1 = manager.add(
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
            3,
            None,
        );
        insta::assert_debug_snapshot!(convert_events(&mut manager));
    }
}
