use crate::game::object::{GameObject, GameObjectId};
use intertrait::CastFrom;
use std::collections::HashMap;
use tearchan_utility::shared::Shared;

pub struct GameObjectOperator<T: ?Sized>
where
    T: CastFrom,
{
    objects: Shared<HashMap<GameObjectId, GameObject<T>>>,
    sorted_object_ids: Shared<Vec<GameObjectId>>,
}

impl<T: ?Sized> GameObjectOperator<T>
where
    T: CastFrom,
{
    pub fn new(
        objects: Shared<HashMap<GameObjectId, GameObject<T>>>,
        sorted_object_ids: Shared<Vec<GameObjectId>>,
    ) -> GameObjectOperator<T> {
        GameObjectOperator {
            objects,
            sorted_object_ids,
        }
    }
}
