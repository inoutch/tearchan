use crate::app::hello_world_scene::HelloWorldScene;
use gfx_hal::window::Extent2D;
use tearchan::core::engine::Engine;
use tearchan::core::engine_config::StartupConfigBuilder;
use tearchan::core::screen::ScreenMode;
use std::env;

pub mod app;

fn main() {
    env::set_var("RUST_LOG", "error");
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
