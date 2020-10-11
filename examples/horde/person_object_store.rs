use nalgebra_glm::Vec2;
use tearchan_horde::object::object_store::ObjectStoreBase;

#[derive(Default)]
pub struct PersonObjectStore {
    position: Vec2,
    rotation: f32,
}

impl ObjectStoreBase for PersonObjectStore {}

impl PersonObjectStore {
    pub fn position(&self) -> &Vec2 {
        &self.position
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }
}
