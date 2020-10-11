use crate::game::object::game_object_base::GameObjectBase;
use crate::game::object::{GameObject, GameObjectId};
use std::collections::HashMap;
use tearchan_utility::shared::Shared;

pub struct GameObjectOperator<T: ?Sized>
where
    T: GameObjectBase,
{
    objects: Shared<HashMap<GameObjectId, GameObject<T>>>,
}

impl<T: ?Sized> GameObjectOperator<T>
where
    T: GameObjectBase,
{
    pub fn new(objects: Shared<HashMap<GameObjectId, GameObject<T>>>) -> GameObjectOperator<T> {
        GameObjectOperator { objects }
    }

    pub fn find_by_id(&self, id: GameObjectId) -> Option<GameObject<T>> {
        self.objects.borrow().get(&id).map(|obj| obj.clone())
    }
}
