use crate::batch::batch_command::{BatchObjectId, BatchProviderCommand};
use crate::batch::batch_object_manager::BatchObjectManager;
use std::collections::HashSet;

pub struct BatchBufferContext<TBuffer> {
    pub buffer: TBuffer,
    pub stride: usize,
}

impl<TBuffer> BatchBufferContext<TBuffer> {
    pub fn new(buffer: TBuffer, stride: usize) -> Self {
        BatchBufferContext { buffer, stride }
    }
}

pub trait BatchProvider {
    fn run(&mut self, command: BatchProviderCommand);

    fn sort(&mut self, ids: Vec<BatchObjectId>) -> HashSet<u32>;

    fn flush(&mut self, batch_object_manager: &mut BatchObjectManager);
}
