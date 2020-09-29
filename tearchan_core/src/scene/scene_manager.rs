use crate::game::game_context::GameContext;
use crate::game::game_plugin_manager::GamePluginManager;
use crate::game::object::game_object_base::GameObjectBase;
use crate::game::object::game_object_manager::GameObjectManager;
use crate::scene::scene_context::SceneContext;
use crate::scene::scene_factory::{SceneFactory, SceneOption};
use crate::scene::scene_result::SceneResult;
use crate::scene::Scene;
use std::option::Option::Some;
use winit::event::WindowEvent;

pub struct SceneManager {
    current_scene: Box<dyn Scene>,
    scene_factory: Option<(SceneFactory, Option<Box<dyn SceneOption>>)>,
    object_manager: GameObjectManager<dyn GameObjectBase>,
}

impl SceneManager {
    pub fn new(scene_creator: SceneFactory) -> SceneManager {
        SceneManager {
            current_scene: Box::new(DummyScene {}),
            scene_factory: Some((scene_creator, None)),
            object_manager: GameObjectManager::new(),
        }
    }

    pub fn event(&mut self, _event: &WindowEvent) {}

    pub fn on_update(
        &mut self,
        context: &mut GameContext,
        plugin_manager: &mut GamePluginManager,
    ) -> bool {
        let mut scene_context =
            SceneContext::new(context, plugin_manager, &mut self.object_manager);
        if let Some((scene_factory, options)) = std::mem::replace(&mut self.scene_factory, None) {
            self.current_scene = scene_factory(&mut scene_context, options);
            self.scene_factory = None;
        }

        match self.current_scene.update(&mut scene_context) {
            SceneResult::Exit => true,
            SceneResult::TransitScene {
                scene_factory,
                option,
            } => {
                self.scene_factory = Some((scene_factory, option));
                false
            }
            _ => false,
        }
    }
}

pub struct DummyScene;

impl Scene for DummyScene {
    fn update(&mut self, _scene_context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
