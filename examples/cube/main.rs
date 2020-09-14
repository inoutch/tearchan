use tearchan::engine::Engine;
use tearchan::engine_config::StartupConfigBuilder;

fn main() {
    let startup_config = StartupConfigBuilder::default()
        .application_name("cube".to_string())
        // .screen_mode(ScreenMode::Windowed {
        //     resolutions: vec![Extent2D {
        //         width: 1200,
        //         height: 800,
        //     }],
        // })
        // .scene_creator(TextScene::creator())
        .build()
        .unwrap();

    Engine::new(startup_config).run();
}
