use crate::cube_scene::CubeScene;
use nalgebra_glm::vec2;
use tearchan::engine::Engine;
use tearchan::engine_config::StartupConfigBuilder;
use tearchan_graphics::screen::ScreenMode;

pub mod cube_scene;

fn main() {
    let startup_config = StartupConfigBuilder::default()
        .application_name("cube".to_string())
        .screen_mode(ScreenMode::Windowed {
            resolutions: vec![vec2(1200, 800)],
        })
        .scene_factory(CubeScene::factory())
        .build()
        .unwrap();

    Engine::new(startup_config).with_default_plugins().run();
}
