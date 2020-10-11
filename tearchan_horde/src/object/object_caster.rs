use crate::object::Object;
use std::rc::Rc;

pub type ObjectCasterType<T> = fn(Rc<dyn Object>) -> Option<Rc<T>>;

pub struct ObjectCaster<T: ?Sized + 'static + Object> {
    entity: fn(Rc<dyn Object>) -> Option<Rc<T>>,
}

impl<T> ObjectCaster<T>
where
    T: ?Sized + 'static + Object,
{
    pub fn new(entity: fn(Rc<dyn Object>) -> Option<Rc<T>>) -> ObjectCaster<T> {
        ObjectCaster { entity }
    }

    pub fn entity(&self) -> &fn(Rc<dyn Object>) -> Option<Rc<T>> {
        &self.entity
    }
}
