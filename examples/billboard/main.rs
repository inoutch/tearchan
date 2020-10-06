use crate::billboard_scene::BillboardScene;
use nalgebra_glm::vec2;
use std::env;
use tearchan::engine::Engine;
use tearchan::engine_config::StartupConfigBuilder;
use tearchan_graphics::screen::ScreenMode;

mod billboard_scene;
mod skeleton_billboard;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let startup_config = StartupConfigBuilder::default()
        .application_name("billboard".to_string())
        .screen_mode(ScreenMode::Windowed {
            resolutions: vec![vec2(1200, 800)],
        })
        .scene_factory(BillboardScene::factory())
        .build()
        .unwrap();

    Engine::new(startup_config).with_default_plugins().run();
}
