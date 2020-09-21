use nalgebra_glm::{vec2, TVec2, Vec2};
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_manager::DummyScene;
use tearchan_graphics::hal::renderer::RendererProperties;
use tearchan_graphics::screen::{ScreenMode, ScreenResolutionMode};

#[derive(Builder)]
#[builder(default)]
pub struct StartupConfig {
    pub application_name: String,
    pub screen_mode: ScreenMode,
    pub screen_size: Option<Vec2>,
    pub screen_resolution_mode: ScreenResolutionMode,
    pub min_physical_size: TVec2<u32>,
    pub scene_factory: SceneFactory,
    pub resource_path: Option<String>,
    pub writable_path: Option<String>,
    pub fps: u64,
    pub renderer_properties: RendererProperties,
}

impl Default for StartupConfig {
    fn default() -> Self {
        StartupConfig {
            application_name: "default".to_string(),
            screen_mode: ScreenMode::FullScreenWindow,
            screen_size: None,
            screen_resolution_mode: ScreenResolutionMode::Auto,
            min_physical_size: vec2(64u32, 64u32),
            scene_factory: |_, _| Box::new(DummyScene {}),
            resource_path: None,
            writable_path: None,
            fps: 60,
            renderer_properties: RendererProperties::default(),
        }
    }
}

pub struct EngineConfig {
    pub application_name: String,
    pub screen_mode: ScreenMode,
    pub screen_size: Option<Vec2>,
    pub screen_resolution_mode: ScreenResolutionMode,
    pub min_physical_size: TVec2<u32>,
    pub scene_factory: SceneFactory,
    pub resource_path: Option<String>,
    pub writable_path: Option<String>,
    pub fps: u64,
    pub renderer_properties: RendererProperties,
}
