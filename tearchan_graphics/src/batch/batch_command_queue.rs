use crate::batch::batch_command::{BatchCommand, BatchObjectId};
use std::sync::{Arc, Mutex};
use tearchan_utility::id_manager::IdGenerator;

pub struct BatchCommandQueue {
    commands: Arc<Mutex<Vec<BatchCommand>>>,
    id_generator: IdGenerator<BatchObjectId>,
}

impl BatchCommandQueue {
    pub fn new(
        commands: Arc<Mutex<Vec<BatchCommand>>>,
        id_generator: IdGenerator<BatchObjectId>,
    ) -> Self {
        BatchCommandQueue {
            commands,
            id_generator,
        }
    }

    pub fn queue(&mut self, mut command: BatchCommand) -> Option<BatchObjectId> {
        let mut next_id = None;
        match &mut command {
            BatchCommand::Add { id, .. } => {
                *id = self.id_generator.gen();
                next_id = Some(*id);
            }
            _ => {}
        }
        self.commands.lock().unwrap().push(command);
        next_id
    }
}
