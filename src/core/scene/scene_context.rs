use crate::core::graphic::hal::backend::RendererApi;
use crate::core::scene::scene_creator::SceneCreator;

pub struct SceneContext<'a, 'b> {
    pub renderer_api: &'a mut RendererApi<'b>,
    commands: &'a mut Vec<SceneContextCommand>,
}

pub enum SceneContextCommand {
    TransitScene { scene_creator: SceneCreator },
}

impl<'a, 'b> SceneContext<'a, 'b> {
    pub fn new(
        renderer_api: &'a mut RendererApi<'b>,
        commands: &'a mut Vec<SceneContextCommand>,
    ) -> SceneContext<'a, 'b> {
        SceneContext {
            renderer_api,
            commands,
        }
    }

    pub fn transit_scene(&mut self, scene_creator: SceneCreator) {
        self.commands
            .push(SceneContextCommand::TransitScene { scene_creator })
    }
}
