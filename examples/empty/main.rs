use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use winit::event::WindowEvent;
use winit::window::WindowBuilder;

struct EmptyScene {}

impl EmptyScene {
    fn factory() -> SceneFactory {
        |_context, _| Box::new(EmptyScene {})
    }
}

impl Scene for EmptyScene {
    fn update(&mut self, _context: &mut SceneContext, _event: WindowEvent) -> SceneControlFlow {
        SceneControlFlow::None
    }

    fn render(&mut self, _context: &mut SceneRenderContext) -> SceneControlFlow {
        SceneControlFlow::None
    }
}

pub fn main() {
    let window_builder = WindowBuilder::new().with_title("empty");
    let startup_config = EngineStartupConfigBuilder::new()
        .window_builder(window_builder)
        .scene_factory(EmptyScene::factory())
        .build();
    let engine = Engine::new(startup_config);
    engine.run();
}
