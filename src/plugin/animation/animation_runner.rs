use crate::plugin::animation::animation_object::AnimationObject;
use std::option::Option::Some;
use tearchan_core::game::game_cast_manager::GameCastManager;
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_object_caster::{GameObjectCaster, GameObjectCasterType};
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;

pub struct AnimationRunner {
    animation_objects: GameObjectManager<dyn AnimationObject>,
    cast_manager: GameCastManager,
}

impl AnimationRunner {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        AnimationRunner {
            animation_objects: GameObjectManager::new(),
            cast_manager: GameCastManager::default(),
        }
    }

    pub fn register(&mut self, caster: GameObjectCasterType<dyn AnimationObject>) {
        self.cast_manager.register(GameObjectCaster::new(caster));
    }
}

impl GamePlugin for AnimationRunner {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(animation_object) = self.cast_manager.cast(game_object) {
            self.animation_objects.add(animation_object);
        }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        self.animation_objects.remove(&game_object.id());
    }

    fn on_update(&mut self, context: &mut GameContext) {
        self.animation_objects
            .for_each_mut(|object| object.borrow_mut().update(context.delta));
    }
}
