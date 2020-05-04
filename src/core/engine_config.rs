use crate::core::scene::scene_creator::SceneCreator;
use crate::core::screen::ScreenMode;
use nalgebra_glm::Vec2;

#[derive(Builder)]
#[builder(default)]
pub struct StartupConfig {
    pub application_name: String,
    pub screen_mode: ScreenMode,
    pub screen_size: Option<Vec2>,
    pub scene_creator: Option<SceneCreator>,
}

impl Default for StartupConfig {
    fn default() -> Self {
        StartupConfig {
            application_name: "default".to_string(),
            screen_mode: ScreenMode::FullScreenWindow,
            screen_size: None,
            scene_creator: None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::engine_config::StartupConfigBuilder;
    use crate::core::scene::scene_base::SceneBase;
    use crate::core::scene::scene_context::SceneContext;

    struct MockScene;
    impl SceneBase for MockScene {
        fn update(&mut self, context: &mut SceneContext, delta: f32) {
            unimplemented!()
        }
    }

    #[test]
    fn test_set_required_values() {
        let config = StartupConfigBuilder::default()
            .application_name("test".to_string())
            .build()
            .unwrap();

        assert_eq!(config.application_name, "test");
    }
}
