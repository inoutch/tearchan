use crate::core::file::File;
use crate::core::graphic::hal::backend::Graphics;
use crate::core::scene::scene_creator::SceneCreator;
use std::any::Any;

pub trait SceneOption {
    fn as_any(&self) -> &dyn Any;
}

pub struct SceneContext<'a, 'b> {
    pub graphics: &'a mut Graphics<'b>,
    pub file: &'a mut File,
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
        graphics: &'a mut Graphics<'b>,
        file: &'a mut File,
        commands: &'a mut Vec<SceneContextCommand>,
    ) -> SceneContext<'a, 'b> {
        SceneContext {
            graphics,
            file,
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
