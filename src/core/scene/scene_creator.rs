use crate::core::scene::scene_base::SceneBase;
use crate::core::scene::scene_context::{SceneContext, SceneOption};

pub type SceneCreator = fn(
    scene_context: &mut SceneContext,
    option: Option<Box<dyn SceneOption>>,
) -> Box<dyn SceneBase>;
