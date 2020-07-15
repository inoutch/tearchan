use gfx_hal::window::Extent2D;
use std::env;
use tearchan::core::engine::Engine;
use tearchan::core::engine_config::StartupConfigBuilder;
use tearchan::core::screen::ScreenMode;
use crate::app::square_scene::SquareScene;

pub mod app;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let startup_config = StartupConfigBuilder::default()
        .application_name("test".to_string())
        .screen_mode(ScreenMode::Windowed {
            resolutions: vec![Extent2D {
                width: 1200,
                height: 800,
            }],
        })
        .scene_creator(SquareScene::creator())
        .build()
        .unwrap();

    Engine::new(startup_config).run();
}
