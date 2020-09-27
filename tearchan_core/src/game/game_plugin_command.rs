use crate::game::object::game_object_base::GameObjectBase;
use crate::game::object::GameObject;

pub enum GamePluginCommand {
    CreateGameObject {
        object: GameObject<dyn GameObjectBase>,
    },
}
