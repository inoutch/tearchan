use crate::scene::scene_factory::{SceneFactory, SceneOption};

pub enum SceneResult {
    None,
    Exit,
    TransitScene {
        scene_factory: SceneFactory,
        option: Option<Box<dyn SceneOption>>,
    },
}
