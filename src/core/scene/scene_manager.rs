use crate::core::graphic::hal::backend::FixedApi;
use crate::core::scene::scene_base::SceneBase;
use crate::core::scene::scene_context::SceneContext;
use crate::core::scene::scene_creator::SceneCreator;

pub struct SceneManager {
    current_scene: Box<dyn SceneBase>,
    scene_creator: Option<SceneCreator>,
}

impl SceneManager {
    pub fn new(scene_creator: SceneCreator) -> SceneManager {
        SceneManager {
            current_scene: Box::new(DummyScene {}),
            scene_creator: Some(scene_creator),
        }
    }

    pub fn render(&mut self, delta: f32, api: &mut FixedApi) {
        let mut scene_context = SceneContext::new(api);
        let scene_creator = std::mem::replace(&mut self.scene_creator, None);
        if let Some(x) = scene_creator {
            self.current_scene = x(&mut scene_context);
            self.scene_creator = None;
        }

        self.current_scene.update(&mut scene_context, delta);
    }
}

struct DummyScene;
impl SceneBase for DummyScene {
    fn update(&mut self, _scene_context: &mut SceneContext, _delta: f32) {}
}
