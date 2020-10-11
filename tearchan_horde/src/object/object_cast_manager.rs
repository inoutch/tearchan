use crate::object::object_caster::ObjectCaster;
use crate::object::Object;
use std::any::Any;
use tearchan_core::game::object::GameObject;

#[derive(Default)]
pub struct ObjectCastManager {
    casters: Vec<Box<dyn Any>>,
}

impl ObjectCastManager {
    pub fn cast<T>(&self, game_object: &GameObject<dyn Object>) -> Option<GameObject<T>>
    where
        T: ?Sized + 'static + Object,
    {
        let target = self.casters.iter().find_map(|x| {
            x.downcast_ref::<ObjectCaster<T>>().and_then(|y| {
                let target_caster = y.entity();
                target_caster(game_object.clone_inner_object())
            })
        })?;
        Some(GameObject::new(target))
    }

    pub fn register<T>(&mut self, caster: ObjectCaster<T>)
    where
        T: ?Sized + 'static + Object,
    {
        self.casters.push(Box::new(caster));
    }

    pub fn casters(&self) -> &Vec<Box<dyn Any>> {
        &self.casters
    }
}
