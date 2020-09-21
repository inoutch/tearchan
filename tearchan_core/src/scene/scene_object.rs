use crate::game::object::game_object_base::GameObjectBase;

pub trait SceneObject: GameObjectBase {
    fn label(&self) -> &str;
}
