use crate::person_object_store::PersonObjectStoreBehavior;
use intertrait::{cast_to, CastFrom};
use nalgebra_glm::{rotate, translate, vec2, vec2_to_vec3, vec3, Mat4, Vec2};
use std::f32::consts::PI;
use std::option::Option::Some;
use std::rc::Rc;
use tearchan::plugin::renderer::standard_2d_renderer::standard_2d_object::Standard2DRenderObject;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::batch::batch_command::{
    BatchCommand, BatchCommandData, BatchCommandTransform, BatchObjectId, BATCH_ID_EMPTY,
};
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_horde::object::object_factory::ObjectFactory;
use tearchan_horde::object::object_store::ObjectStore;
use tearchan_horde::object::Object;
use tearchan_utility::mesh::MeshBuilder;

pub struct PersonObject {
    store: ObjectStore<dyn PersonObjectStoreBehavior>,
    batch_queue: Option<BatchCommandQueue>,
    batch_object_id: Option<BatchObjectId>,
}

pub trait PersonBehavior: CastFrom {
    fn set_position(&mut self, position: Vec2);
    fn set_rotation(&mut self, angle: f32);
    fn update_transform(&mut self);
}

#[cast_to]
impl GameObjectBase for PersonObject {}

#[cast_to]
impl Object for PersonObject {}

#[cast_to]
impl Standard2DRenderObject for PersonObject {
    fn attach_queue(&mut self, mut queue: BatchCommandQueue) {
        let mesh = MeshBuilder::new()
            .with_square(vec2(100.0f32, 100.0f32))
            .build()
            .unwrap();
        let batch_object_id = queue
            .queue(BatchCommand::Add {
                id: BATCH_ID_EMPTY,
                data: vec![
                    BatchCommandData::V1U32 {
                        data: mesh.indices.clone(),
                    },
                    BatchCommandData::V3F32 {
                        data: mesh.positions.clone(),
                    },
                    BatchCommandData::V4F32 {
                        data: mesh.colors.clone(),
                    },
                    BatchCommandData::V2F32 {
                        data: mesh.texcoords,
                    },
                ],
                order: None,
            })
            .unwrap();

        self.batch_object_id = Some(batch_object_id);
        self.batch_queue = Some(queue);
    }
}

impl PersonObject {
    pub fn factory() -> ObjectFactory {
        |(store, _, _)| {
            let store = store.cast()?;
            Some(GameObject::new(Rc::new(PersonObject::new(store))))
        }
    }

    pub fn kind() -> &'static str {
        "person"
    }

    pub fn new(store: ObjectStore<dyn PersonObjectStoreBehavior>) -> Self {
        PersonObject {
            store,
            batch_queue: None,
            batch_object_id: None,
        }
    }
}

#[cast_to]
impl PersonBehavior for PersonObject {
    fn set_position(&mut self, position: Vec2) {
        self.store.borrow_mut().set_position(position);
    }

    fn set_rotation(&mut self, rotation: f32) {
        self.store.borrow_mut().set_rotation(rotation);
    }

    fn update_transform(&mut self) {
        if let Some(queue) = &mut self.batch_queue {
            queue.queue(BatchCommand::Transform {
                id: self.batch_object_id.unwrap(),
                attribute: 1,
                transform: BatchCommandTransform::Mat4 {
                    m: rotate(
                        &translate(
                            &Mat4::identity(),
                            &vec2_to_vec3(self.store.borrow().position()),
                        ),
                        self.store.borrow().rotation() / 180.0f32 * PI,
                        &vec3(0.0f32, 0.0f32, 1.0f32),
                    ),
                },
            });
        }
    }
}
