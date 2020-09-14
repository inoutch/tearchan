use crate::renderer::render_command::RenderCommand;
use crate::renderer::RenderId;
use std::sync::{Arc, Mutex};
use tearchan_utility::id_manager::IdGenerator;

pub struct RenderCommandQueue {
    commands: Arc<Mutex<Vec<RenderCommand>>>,
    id_generator: IdGenerator<RenderId>,
}

impl RenderCommandQueue {
    pub fn new(
        commands: Arc<Mutex<Vec<RenderCommand>>>,
        id_generator: IdGenerator<RenderId>,
    ) -> Self {
        RenderCommandQueue {
            commands,
            id_generator,
        }
    }

    pub fn queue(&mut self, mut command: RenderCommand) -> Option<RenderId> {
        let mut next_id = None;
        match &mut command {
            RenderCommand::Add { id, .. } => {
                *id = self.id_generator.gen();
                next_id = Some(*id);
            }
            _ => {}
        }
        self.commands.lock().unwrap().push(command);
        next_id
    }
}
