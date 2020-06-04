use crate::core::file::file_api::FileApi;
use crate::core::graphic::hal::backend::RendererApi;
use crate::core::scene::scene_creator::SceneCreator;
use std::any::Any;

pub trait SceneOption {
    fn as_any(&self) -> &dyn Any;
}

pub struct SceneContext<'a, 'b> {
    pub renderer_api: &'a mut RendererApi<'b>,
    pub file_api: &'a mut FileApi,
    commands: &'a mut Vec<SceneContextCommand>,
}

pub enum SceneContextCommand {
    TransitScene {
        scene_creator: SceneCreator,
        option: Option<Box<dyn SceneOption>>,
    },
}

impl<'a, 'b> SceneContext<'a, 'b> {
    pub fn new(
        renderer_api: &'a mut RendererApi<'b>,
        file_api: &'a mut FileApi,
        commands: &'a mut Vec<SceneContextCommand>,
    ) -> SceneContext<'a, 'b> {
        SceneContext {
            renderer_api,
            file_api,
            commands,
        }
    }

    pub fn transit_scene(
        &mut self,
        scene_creator: SceneCreator,
        option: Option<Box<dyn SceneOption>>,
    ) {
        self.commands.push(SceneContextCommand::TransitScene {
            scene_creator,
            option,
        })
    }
}
