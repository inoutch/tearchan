use crate::core::scene::scene_base::SceneBase;
use crate::core::scene::scene_context::SceneContext;

pub type SceneCreator = fn(scene_context: &mut SceneContext) -> Box<dyn SceneBase>;
