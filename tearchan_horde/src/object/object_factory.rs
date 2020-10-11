use crate::object::object_store::{ObjectStore, ObjectStoreBase};
use crate::object::Object;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_operator::GameObjectOperator;
use tearchan_core::game::object::GameObject;

pub type ObjectFactory =
    fn(properties: ObjectFactoryProperties) -> Option<GameObject<dyn GameObjectBase>>;

pub type ObjectFactoryProperties = (
    ObjectStore<dyn ObjectStoreBase>,
    Option<GameObject<dyn Object>>,
    GameObjectOperator<dyn Object>,
);
