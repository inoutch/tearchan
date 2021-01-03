use crate::scene::context::SceneContext;
use crate::scene::Scene;
use std::any::Any;

pub trait SceneOption: Any {}

pub type SceneFactory =
    fn(scene_context: &mut SceneContext, option: Option<Box<dyn SceneOption>>) -> Box<dyn Scene>;
