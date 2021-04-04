use crate::scene::factory::SceneFactory;
use std::any::Any;
use winit::window::WindowBuilder;

pub struct EngineStartupConfig {
    pub window_builder: WindowBuilder,
    pub fps: u64,
    // scene
    pub scene_factory: SceneFactory,
    // custom context
    pub custom: Option<Box<dyn Any>>,
}

pub struct EngineStartupConfigBuilder<TWindowBuilder, TSceneFactory> {
    pub window_builder: TWindowBuilder,
    pub fps: u64,
    // scene
    pub scene_factory: TSceneFactory,
    // custom context
    pub custom: Option<Box<dyn Any>>,
}

impl EngineStartupConfigBuilder<(), ()> {
    pub fn new() -> Self {
        EngineStartupConfigBuilder {
            window_builder: (),
            fps: 60,
            scene_factory: (),
            custom: None,
        }
    }
}

impl<TWindowBuilder, TSceneFactory> EngineStartupConfigBuilder<TWindowBuilder, TSceneFactory> {
    pub fn window_builder(
        self,
        window_builder: WindowBuilder,
    ) -> EngineStartupConfigBuilder<WindowBuilder, TSceneFactory> {
        EngineStartupConfigBuilder {
            window_builder,
            fps: self.fps,
            scene_factory: self.scene_factory,
            custom: self.custom,
        }
    }

    pub fn fps(self, fps: u64) -> EngineStartupConfigBuilder<TWindowBuilder, TSceneFactory> {
        EngineStartupConfigBuilder {
            window_builder: self.window_builder,
            fps,
            scene_factory: self.scene_factory,
            custom: self.custom,
        }
    }

    pub fn scene_factory(
        self,
        scene_factory: SceneFactory,
    ) -> EngineStartupConfigBuilder<TWindowBuilder, SceneFactory> {
        EngineStartupConfigBuilder {
            window_builder: self.window_builder,
            fps: self.fps,
            scene_factory,
            custom: self.custom,
        }
    }

    pub fn custom(
        self,
        custom: Box<dyn Any>,
    ) -> EngineStartupConfigBuilder<TWindowBuilder, TSceneFactory> {
        EngineStartupConfigBuilder {
            window_builder: self.window_builder,
            fps: self.fps,
            scene_factory: self.scene_factory,
            custom: Some(custom),
        }
    }
}

impl EngineStartupConfigBuilder<WindowBuilder, SceneFactory> {
    pub fn build(self) -> EngineStartupConfig {
        EngineStartupConfig {
            window_builder: self.window_builder,
            fps: self.fps,
            scene_factory: self.scene_factory,
            custom: self.custom,
        }
    }
}

impl EngineStartupConfig {
    pub fn new_with_title(title: &str, scene_factory: SceneFactory) -> Self {
        let window_builder = WindowBuilder::new().with_title(title);
        EngineStartupConfigBuilder::new()
            .window_builder(window_builder)
            .scene_factory(scene_factory)
            .build()
    }
}
