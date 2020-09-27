use intertrait::cast_to;
use nalgebra_glm::Vec2;
use tearchan_horde::object::object_store::ObjectStoreBase;

#[derive(Default)]
pub struct PersonObjectStore {
    position: Vec2,
    rotation: f32,
}

pub trait PersonObjectStoreBehavior: ObjectStoreBase {
    fn position(&self) -> &Vec2;
    fn rotation(&self) -> f32;
    fn set_position(&mut self, position: Vec2);
    fn set_rotation(&mut self, rotation: f32);
}

impl ObjectStoreBase for PersonObjectStore {}

#[cast_to]
impl PersonObjectStoreBehavior for PersonObjectStore {
    fn position(&self) -> &Vec2 {
        &self.position
    }

    fn rotation(&self) -> f32 {
        self.rotation
    }

    fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }
}
