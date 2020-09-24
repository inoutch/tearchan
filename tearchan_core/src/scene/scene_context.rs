use crate::game::game_context::GameContext;
use crate::game::game_plugin_manager::GamePluginManager;
use crate::game::object::game_object_base::GameObjectBase;
use crate::game::object::game_object_manager::GameObjectManager;
use crate::game::object::{GameObject, GameObjectId};

pub struct SceneContext<'a, 'b> {
    pub g: &'b mut GameContext<'a>,
    plugin_manager: &'b mut GamePluginManager,
    object_manager: &'b mut GameObjectManager<dyn GameObjectBase>,
}

impl<'a, 'b> SceneContext<'a, 'b> {
    pub fn new(
        game_context: &'b mut GameContext<'a>,
        plugin_manager: &'b mut GamePluginManager,
        object_manager: &'b mut GameObjectManager<dyn GameObjectBase>,
    ) -> SceneContext<'a, 'b> {
        SceneContext {
            g: game_context,
            plugin_manager,
            object_manager,
        }
    }

    pub fn add(&mut self, game_object: GameObject<dyn GameObjectBase>) {
        self.plugin_manager.for_each_mut(|plugin| {
            plugin.on_add(&game_object);
        });
        self.object_manager.add(game_object);
    }

    pub fn remove(&mut self, id: &GameObjectId) -> Option<GameObject<dyn GameObjectBase>> {
        let game_object = self.object_manager.remove(id)?;
        self.plugin_manager.for_each_mut(|plugin| {
            plugin.on_remove(&game_object);
        });
        Some(game_object)
    }

    pub fn plugin_manager_mut(&mut self) -> &mut GamePluginManager {
        self.plugin_manager
    }
}
