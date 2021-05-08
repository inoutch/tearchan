use crate::scene::context::SceneContext;
use crate::scene::Scene;
use std::any::Any;

pub trait SceneOption: Any {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type SceneFactory =
    fn(scene_context: &mut SceneContext, option: Option<Box<dyn SceneOption>>) -> Box<dyn Scene>;
