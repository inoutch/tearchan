use crate::batch::batch_command::{BatchCommand, BatchObjectId};
use std::sync::mpsc::Sender;
use tearchan_utility::id_manager::IdGenerator;

pub struct BatchCommandQueue {
    sender: Sender<BatchCommand>,
    id_generator: IdGenerator<BatchObjectId>,
}

impl BatchCommandQueue {
    pub fn new(sender: Sender<BatchCommand>, id_generator: IdGenerator<BatchObjectId>) -> Self {
        BatchCommandQueue {
            sender,
            id_generator,
        }
    }

    pub fn queue(&mut self, mut command: BatchCommand) -> Option<BatchObjectId> {
        let mut next_id = None;
        if let BatchCommand::Add { id, .. } = &mut command {
            *id = self.id_generator.gen();
            next_id = Some(*id);
        }
        self.sender.send(command).unwrap();
        next_id
    }
}
