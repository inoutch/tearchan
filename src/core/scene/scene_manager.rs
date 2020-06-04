use crate::core::file::file_api::FileApi;
use crate::core::graphic::hal::backend::RendererApi;
use crate::core::scene::scene_base::SceneBase;
use crate::core::scene::scene_context::{SceneContext, SceneContextCommand, SceneOption};
use crate::core::scene::scene_creator::SceneCreator;

pub struct SceneManager {
    current_scene: Box<dyn SceneBase>,
    scene_creator: Option<(SceneCreator, Option<Box<dyn SceneOption>>)>,
    commands: Vec<SceneContextCommand>,
}

impl SceneManager {
    pub fn new(scene_creator: SceneCreator) -> SceneManager {
        SceneManager {
            current_scene: Box::new(DummyScene {}),
            scene_creator: Some((scene_creator, None)),
            commands: vec![],
        }
    }

    pub fn render(&mut self, delta: f32, renderer_api: &mut RendererApi, file_api: &mut FileApi) {
        while !self.commands.is_empty() {
            if let Some(command) = self.commands.pop() {
                match command {
                    SceneContextCommand::TransitScene {
                        scene_creator,
                        option,
                    } => {
                        self.scene_creator = Some((scene_creator, option));
                    }
                }
            }
        }

        let mut scene_context = SceneContext::new(renderer_api, file_api, &mut self.commands);
        let scene_creator = std::mem::replace(&mut self.scene_creator, None);
        if let Some(x) = scene_creator {
            self.current_scene = x.0(&mut scene_context, x.1);
            self.scene_creator = None;
        }

        self.current_scene.update(&mut scene_context, delta);
    }
}

pub struct DummyScene;

impl SceneBase for DummyScene {
    fn update(&mut self, _scene_context: &mut SceneContext, _delta: f32) {}
}
