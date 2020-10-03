use tearchan_core::game::object::game_object_base::GameObjectBase;

pub trait AnimationObject: GameObjectBase {
    fn update(&mut self, delta: f32);
}
