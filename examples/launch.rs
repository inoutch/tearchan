use crate::app::hello_world_scene::HelloWorldScene;
use gfx_hal::window::Extent2D;
use std::env;
use tearchan::core::engine::Engine;
use tearchan::core::engine_config::StartupConfigBuilder;
use tearchan::core::screen::ScreenMode;

pub mod app;
pub mod texture_bundle;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let startup_config = StartupConfigBuilder::default()
        .application_name("test".to_string())
        .screen_mode(ScreenMode::Windowed {
            resolutions: vec![Extent2D {
                width: 600,
                height: 400,
            }],
        })
        .scene_creator(HelloWorldScene::creator())
        .build()
        .unwrap();

    Engine::new(startup_config).run();
}
