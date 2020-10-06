use crate::horde_scene::HordeScene;
use nalgebra_glm::vec2;
use std::env;
use tearchan::engine::Engine;
use tearchan::engine_config::StartupConfigBuilder;
use tearchan_graphics::screen::ScreenMode;

pub mod horde_provider;
pub mod horde_scene;
pub mod person_object;
pub mod person_object_factory;
pub mod person_object_store;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let startup_config = StartupConfigBuilder::default()
        .application_name("horde".to_string())
        .screen_mode(ScreenMode::Windowed {
            resolutions: vec![vec2(1200, 800)],
        })
        .fps(144)
        .scene_factory(HordeScene::factory())
        .build()
        .unwrap();

    Engine::new(startup_config).with_default_plugins().run();
}
