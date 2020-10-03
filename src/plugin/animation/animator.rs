use crate::plugin::animation::animation_object::AnimationObject;
use std::option::Option::Some;
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;

pub struct Animator {
    animation_objects: GameObjectManager<dyn AnimationObject>,
}

impl Animator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Animator {
            animation_objects: GameObjectManager::new(),
        }
    }
}

impl GamePlugin for Animator {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(animation_object) = game_object.cast::<dyn AnimationObject>() {
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
