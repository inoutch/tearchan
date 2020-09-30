use crate::batch::batch_command::{BatchCommand, BatchCommandTransform, BatchObjectId};
use crate::batch::batch_object::BatchObject;
use std::collections::{HashMap, HashSet};
use std::option::Option::Some;
use tearchan_utility::btree::DuplicatableBTreeMap;

pub struct BatchObjectManager {
    objects: HashMap<BatchObjectId, BatchObject>,
    sorted_object_ids: DuplicatableBTreeMap<i32, BatchObjectId>,
    changed_object_map: HashMap<BatchObjectId, HashSet<u32>>,
}

impl BatchObjectManager {
    #[allow(clippy::new_without_default)]
    pub fn new() -> BatchObjectManager {
        BatchObjectManager {
            objects: HashMap::new(),
            sorted_object_ids: DuplicatableBTreeMap::new(),
            changed_object_map: HashMap::new(),
        }
    }

    pub fn run(&mut self, command: BatchCommand) {
        match command {
            BatchCommand::Add { id, data, order } => {
                let order = order.map_or(0, |x| x);
                let transforms = vec![BatchCommandTransform::None; data.len()];
                let mut set = HashSet::with_capacity(data.len());
                for i in 0u32..data.len() as u32 {
                    set.insert(i);
                }

                self.objects.insert(
                    id,
                    BatchObject {
                        id,
                        data,
                        transforms,
                        order,
                    },
                );
                self.sorted_object_ids.push_back(order, id);
                self.changed_object_map.insert(id, set);
            }
            BatchCommand::Remove { id } => {
                self.changed_object_map.remove(&id);
                if let Some(object) = self.objects.remove(&id) {
                    self.sorted_object_ids.remove(&object.order, &id);
                }
            }
            BatchCommand::Transform {
                id,
                attribute,
                transform,
            } => {
                let object = match self.objects.get_mut(&id) {
                    Some(object) => object,
                    None => return,
                };
                object.transforms[attribute as usize] = transform;

                match self.changed_object_map.get_mut(&id) {
                    None => {
                        let mut set = HashSet::new();
                        set.insert(attribute);
                        self.changed_object_map.insert(id, set);
                    }
                    Some(set) => {
                        set.insert(attribute);
                    }
                }
            }
            BatchCommand::Replace {
                id,
                attribute,
                data,
            } => {
                let object = match self.objects.get_mut(&id) {
                    Some(object) => object,
                    None => return,
                };
                object.data[attribute as usize] = data;

                match self.changed_object_map.get_mut(&id) {
                    None => {
                        let mut set = HashSet::new();
                        set.insert(attribute);
                        self.changed_object_map.insert(id, set);
                    }
                    Some(set) => {
                        set.insert(attribute);
                    }
                }
            }
            BatchCommand::Refresh { attribute } => {
                for id in self.objects.keys() {
                    match self.changed_object_map.get_mut(id) {
                        None => {
                            let mut set = HashSet::new();
                            set.insert(attribute);
                            self.changed_object_map.insert(*id, set);
                        }
                        Some(set) => {
                            set.insert(attribute);
                        }
                    }
                }
            }
        }
    }

    pub fn flush<F>(&mut self, mut callback: F)
    where
        F: FnMut(&BatchObject, u32),
    {
        for (id, attributes) in &mut self.changed_object_map {
            let object = match self.objects.get(&id) {
                Some(object) => object,
                None => continue,
            };
            for attribute in attributes.iter() {
                callback(object, *attribute);
            }
        }
        self.changed_object_map.clear();
    }

    pub fn create_sorted_ids(&self) -> Vec<BatchObjectId> {
        self.sorted_object_ids
            .iter()
            .map(|(_, ids)| ids)
            .flatten()
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::batch::batch_command::{BatchCommand, BatchCommandData};
    use crate::batch::batch_object_manager::BatchObjectManager;
    use nalgebra_glm::{vec2, vec3};
    use tearchan_utility::math::vec::vec4_white;

    #[test]
    fn test() {
        let mut manager = BatchObjectManager::new();
        manager.run(BatchCommand::Add {
            id: 0,
            data: vec![
                BatchCommandData::V3U32 {
                    data: vec![vec3(0, 1, 2)],
                }, // index
                BatchCommandData::V3F32 {
                    data: vec![
                        vec3(0.0f32, 0.0f32, 0.0f32),
                        vec3(1.0f32, 0.0f32, 0.0f32),
                        vec3(1.0f32, 1.0f32, 0.0f32),
                    ],
                }, // position
                BatchCommandData::V2F32 {
                    data: vec![
                        vec2(0.0f32, 1.0f32),
                        vec2(1.0f32, 1.0f32),
                        vec2(1.0f32, 0.0f32),
                    ],
                }, // uv
                BatchCommandData::V4F32 {
                    data: vec![vec4_white(), vec4_white(), vec4_white()],
                }, // color
            ],
            order: Some(0),
        });

        let mut indices = vec![];
        let mut positions = vec![];
        let mut texcoords = vec![];
        let mut colors = vec![];
        manager.flush(|object, attribute| match attribute {
            0 => {
                object.for_each_v3u32(0, |i, v| {
                    assert_eq!(i, indices.len());
                    (&mut indices).push(v);
                });
            }
            1 => {
                object.for_each_v3f32(1, |i, v| {
                    assert_eq!(i, positions.len());
                    (&mut positions).push(v);
                });
            }
            2 => {
                object.for_each_v2f32(2, |i, v| {
                    assert_eq!(i, texcoords.len());
                    (&mut texcoords).push(v);
                });
            }
            3 => {
                object.for_each_v4f32(3, |i, v| {
                    assert_eq!(i, colors.len());
                    (&mut colors).push(v);
                });
            }
            _ => {}
        });

        assert_eq!(indices, [0, 1, 2]);
        assert_eq!(
            positions,
            [0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32, 0.0f32]
        );
        assert_eq!(texcoords, [0.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 0.0f32]);
        assert_eq!(
            colors,
            [
                1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32,
                1.0f32, 1.0f32
            ]
        );
    }
}
