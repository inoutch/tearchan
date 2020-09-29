use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;

pub struct UIScene {}

impl UIScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| Box::new(UIScene {})
    }
}

impl Scene for UIScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::Exit
    }
}
