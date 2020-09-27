use crate::batch::batch_command::{BatchCommand, BatchCommandTransform, BatchObjectId};
use crate::batch::batch_object::BatchObject;
use std::collections::{HashMap, HashSet};
use std::option::Option::Some;
use tearchan_utility::btree::DuplicatableBTreeMap;

pub struct BatchObjectManager {
    objects: HashMap<BatchObjectId, BatchObject>,
    changed_objects: DuplicatableBTreeMap<i32, BatchObjectId>,
    changed_object_set: HashSet<BatchObjectId>,
}

impl BatchObjectManager {
    pub fn new() -> BatchObjectManager {
        BatchObjectManager {
            objects: HashMap::new(),
            changed_objects: DuplicatableBTreeMap::new(),
            changed_object_set: HashSet::new(),
        }
    }

    pub fn run(&mut self, command: BatchCommand) {
        match command {
            BatchCommand::Add { id, data, order } => {
                let order = order.map_or(0, |x| x);
                let transforms = vec![BatchCommandTransform::None; data.len()];
                let object = BatchObject {
                    id,
                    data,
                    transforms,
                    order,
                };
                self.objects.insert(id, object);
                self.changed_objects.push_back(order, id);
                self.changed_object_set.insert(id);
            }
            BatchCommand::Remove { id } => {
                self.objects.remove(&id);
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

                if !self.changed_object_set.contains(&id) {
                    self.changed_object_set.insert(id);
                    self.changed_objects.push_back(object.order, id);
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
            }
            BatchCommand::CopyForEach { .. } => unimplemented!(),
        }
    }

    pub fn flush<F>(&mut self, mut callback: F)
    where
        F: FnMut(&BatchObject),
    {
        for (_, range) in self.changed_objects.range_mut(..) {
            while let Some(id) = range.pop_front() {
                self.changed_object_set.remove(&id);

                let object = match self.objects.get(&id) {
                    Some(object) => object,
                    None => continue,
                };
                callback(object);
            }
        }
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
        manager.flush(|object| {
            object.for_each_v3u32(0, |i, v| {
                assert_eq!(i, indices.len());
                (&mut indices).push(v);
            });
            object.for_each_v3f32(1, |i, v| {
                assert_eq!(i, positions.len());
                (&mut positions).push(v);
            });
            object.for_each_v2f32(2, |i, v| {
                assert_eq!(i, texcoords.len());
                (&mut texcoords).push(v);
            });
            object.for_each_v4f32(3, |i, v| {
                assert_eq!(i, colors.len());
                (&mut colors).push(v);
            });
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
