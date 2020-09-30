use crate::batch::batch_command::{BatchCommand, BatchObjectId, BatchProviderCommand};
use crate::batch::batch_command_queue::BatchCommandQueue;
use crate::batch::batch_object_manager::BatchObjectManager;
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
    batch_object_manager: BatchObjectManager,
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
            batch_object_manager: BatchObjectManager::new(),
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
        let mut need_sort_all = false;
        while let Some(command) = self.commands.lock().unwrap().pop_front() {
            self.provider.run(match &command {
                BatchCommand::Add { id, data, order } => {
                    if order.is_some() {
                        need_sort_all = true;
                    }
                    BatchProviderCommand::Add { id: *id, data }
                }
                BatchCommand::Remove { id } => BatchProviderCommand::Remove { id: *id },
                BatchCommand::Transform { .. } => BatchProviderCommand::None,
                BatchCommand::Replace {
                    id,
                    attribute,
                    data,
                } => BatchProviderCommand::Replace {
                    id: *id,
                    attribute: *attribute,
                    data,
                },
                _ => BatchProviderCommand::None,
            });
            self.batch_object_manager.run(command);
        }

        if need_sort_all {
            let attributes = self
                .provider
                .sort(self.batch_object_manager.create_sorted_ids());
            for attribute in attributes {
                self.batch_object_manager
                    .run(BatchCommand::Refresh { attribute });
            }
        }

        self.provider.flush(&mut self.batch_object_manager);
    }
}
