use crate::batch::batch_command::{BatchCommand, BatchObjectId};
use crate::batch::batch_command_queue::BatchCommandQueue;
use crate::batch::batch_provider::BatchProvider;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tearchan_utility::id_manager::IdManager;

pub mod batch_buffer;
pub mod batch_command;
pub mod batch_command_queue;
pub mod batch_object;
pub mod batch_object_manager;
pub mod batch_pointer;
pub mod batch_provider;

pub type BatchIndex = u32;

pub const DEFAULT_DEFRAGMENTATION_BORDER: usize = 10000;

pub struct BatchContext {
    pub draw_order: BatchIndex,
}

pub struct Batch<T: BatchProvider> {
    provider: T,
    commands: Arc<Mutex<VecDeque<BatchCommand>>>,
    id_manager: IdManager<BatchObjectId>,
}

impl<T> Batch<T>
where
    T: BatchProvider,
{
    pub fn new(provider: T) -> Self {
        Batch {
            provider,
            commands: Arc::new(Mutex::new(VecDeque::new())),
            id_manager: IdManager::new(0u64, |id| id + 1),
        }
    }

    pub fn provider(&self) -> &T {
        &self.provider
    }

    pub fn create_queue(&mut self) -> BatchCommandQueue {
        BatchCommandQueue::new(
            Arc::clone(&self.commands),
            self.id_manager.create_generator(),
        )
    }

    pub fn flush(&mut self) {
        while let Some(command) = self.commands.lock().unwrap().pop_front() {
            self.provider.run(command);
        }
        self.provider.flush()
    }
}
