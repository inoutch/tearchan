use crate::game::object::game_object_base::GameObjectBase;
use std::rc::Rc;

pub type GameObjectCasterType<T> = fn(Rc<dyn GameObjectBase>) -> Option<Rc<T>>;

pub struct GameObjectCaster<T: ?Sized + 'static + GameObjectBase> {
    entity: fn(Rc<dyn GameObjectBase>) -> Option<Rc<T>>,
}

impl<T> GameObjectCaster<T>
where
    T: ?Sized + 'static + GameObjectBase,
{
    pub fn new(entity: fn(Rc<dyn GameObjectBase>) -> Option<Rc<T>>) -> GameObjectCaster<T> {
        GameObjectCaster { entity }
    }

    pub fn entity(&self) -> &fn(Rc<dyn GameObjectBase>) -> Option<Rc<T>> {
        &self.entity
    }
}
