use crate::game::game_context::GameContext;
use crate::game::game_plugin::GamePlugin;
use crate::game::object::game_object_base::GameObjectBase;
use crate::game::object::game_object_manager::GameObjectManager;
use crate::game::object::{GameObject, GameObjectId};

pub struct SceneContext<'a, 'b> {
    _game_context: &'b mut GameContext<'a>,
    plugins: &'b mut Vec<Box<dyn GamePlugin>>,
    object_manager: &'b mut GameObjectManager<dyn GameObjectBase>,
}

impl<'a, 'b> SceneContext<'a, 'b> {
    pub fn new(
        game_context: &'b mut GameContext<'a>,
        plugins: &'b mut Vec<Box<dyn GamePlugin>>,
        object_manager: &'b mut GameObjectManager<dyn GameObjectBase>,
    ) -> SceneContext<'a, 'b> {
        SceneContext {
            _game_context: game_context,
            plugins,
            object_manager,
        }
    }

    pub fn add(&mut self, game_object: GameObject<dyn GameObjectBase>) {
        for plugin in self.plugins.iter_mut() {
            plugin.on_add(&game_object);
        }
        self.object_manager.add(game_object);
    }

    pub fn remove(&mut self, id: &GameObjectId) -> Option<GameObject<dyn GameObjectBase>> {
        let game_object = self.object_manager.remove(id)?;
        for plugin in self.plugins.iter_mut() {
            plugin.on_remove(&game_object);
        }
        Some(game_object)
    }
}
