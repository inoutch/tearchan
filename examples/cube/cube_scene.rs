use std::rc::Rc;
use tearchan_core::game::object::GameObject;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;

pub struct CubeScene {}

impl CubeScene {
    pub fn factory() -> SceneFactory {
        |_, _| Box::new(CubeScene {})
    }
}

impl Scene for CubeScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
