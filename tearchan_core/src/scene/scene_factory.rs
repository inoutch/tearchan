use crate::scene::scene_context::SceneContext;
use crate::scene::Scene;
use tearchan_utility::any::AsAny;

pub trait SceneOption: AsAny {}

pub type SceneFactory =
    fn(scene_context: &mut SceneContext, option: Option<Box<dyn SceneOption>>) -> Box<dyn Scene>;
