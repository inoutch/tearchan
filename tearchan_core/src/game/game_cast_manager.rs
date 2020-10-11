use crate::game::game_object_caster::GameObjectCaster;
use crate::game::object::game_object_base::GameObjectBase;
use crate::game::object::GameObject;
use std::any::Any;

#[derive(Default)]
pub struct GameCastManager {
    casters: Vec<Box<dyn Any>>,
}

impl GameCastManager {
    pub fn cast<T>(&self, game_object: &GameObject<dyn GameObjectBase>) -> Option<GameObject<T>>
    where
        T: ?Sized + 'static + GameObjectBase,
    {
        let target = self.casters.iter().find_map(|x| {
            x.downcast_ref::<GameObjectCaster<T>>().and_then(|y| {
                let target_caster = y.entity();
                target_caster(game_object.clone_inner_object())
            })
        })?;
        Some(GameObject::new(target))
    }

    pub fn register<T>(&mut self, caster: GameObjectCaster<T>)
    where
        T: ?Sized + 'static + GameObjectBase,
    {
        self.casters.push(Box::new(caster));
    }

    pub fn casters(&self) -> &Vec<Box<dyn Any>> {
        &self.casters
    }
}
