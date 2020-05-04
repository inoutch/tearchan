use crate::core::scene::scene_context::SceneContext;

pub trait SceneBase {
    fn update(&mut self, context: &mut SceneContext, delta: f32);
}
