use crate::ui_scene::UIScene;
use nalgebra_glm::vec2;
use std::env;
use tearchan::engine::Engine;
use tearchan::engine_config::StartupConfigBuilder;
use tearchan_graphics::screen::ScreenMode;

pub mod ui_scene;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let startup_config = StartupConfigBuilder::default()
        .application_name("cube".to_string())
        .screen_mode(ScreenMode::Windowed {
            resolutions: vec![vec2(1200, 800)],
        })
        .scene_factory(UIScene::factory())
        .fps(144)
        .build()
        .unwrap();

    Engine::new(startup_config).with_default_plugins().run();
}
