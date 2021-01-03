use nalgebra_glm::vec2;
use tearchan::engine::Engine;
use tearchan::engine_config::StartupConfigBuilder;
use tearchan_graphics::screen::ScreenMode;

#[cfg(not(target_arch = "wasm32"))]
use std::env;

use crate::quad_scene::QuadScene;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod quad_scene;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env::set_var("RUST_LOG", "info");
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Debug).unwrap();

    let startup_config = StartupConfigBuilder::default()
        .application_name("quad".to_string())
        .screen_mode(ScreenMode::Windowed {
            resolutions: vec![vec2(1200, 800)],
        })
        .scene_factory(QuadScene::factory())
        .build()
        .unwrap();

    Engine::new(startup_config).with_default_plugins().run();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    main();
}
