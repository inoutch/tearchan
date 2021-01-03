use crate::scene::factory::SceneFactory;
use winit::window::WindowBuilder;

#[derive(Builder)]
pub struct EngineStartupConfig {
    pub window_builder: WindowBuilder,
    #[builder(default = "60")]
    pub fps: u64,
    // scene
    pub scene_factory: SceneFactory,
}

impl EngineStartupConfig {
    pub fn new_with_title(title: &str, scene_factory: SceneFactory) -> Self {
        let window_builder = WindowBuilder::new().with_title(title);
        EngineStartupConfigBuilder::default()
            .window_builder(window_builder)
            .scene_factory(scene_factory)
            .build()
            .unwrap()
    }
}
