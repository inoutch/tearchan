use crate::plugin::renderer::standard_font_renderer::standard_font_command::StandardFontCommand;
use std::sync::mpsc::Sender;
use tearchan_core::game::object::EMPTY_ID;
use tearchan_graphics::batch::batch_command::{BatchCommand, BatchCommandData, BatchObjectId};
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;

pub struct StandardFontCommandQueue {
    command_queue: BatchCommandQueue,
    sender: Sender<StandardFontCommand>,
}

impl StandardFontCommandQueue {
    pub fn new(
        command_queue: BatchCommandQueue,
        sender: Sender<StandardFontCommand>,
    ) -> StandardFontCommandQueue {
        StandardFontCommandQueue {
            command_queue,
            sender,
        }
    }

    pub fn create_text(&mut self, text: String, order: Option<i32>) -> BatchObjectId {
        let id = self
            .command_queue
            .queue(BatchCommand::Add {
                id: EMPTY_ID,
                data: vec![
                    BatchCommandData::V1U32 { data: vec![] },
                    BatchCommandData::V3F32 { data: vec![] },
                    BatchCommandData::V4F32 { data: vec![] },
                    BatchCommandData::V2F32 { data: vec![] },
                ],
                order,
            })
            .unwrap();
        self.sender
            .send(StandardFontCommand::SetText { id, text })
            .unwrap();
        id
    }

    pub fn update_text(&mut self, id: BatchObjectId, text: String) {
        self.sender
            .send(StandardFontCommand::SetText { id, text })
            .unwrap();
    }

    pub fn destroy_text(&mut self, id: BatchObjectId) {
        self.command_queue.queue(BatchCommand::Remove { id });
    }

    #[inline]
    pub fn queue(&mut self, command: BatchCommand) -> Option<BatchObjectId> {
        self.command_queue.queue(command)
    }
}
