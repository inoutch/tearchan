use crate::batch::batch_command::BatchCommand;

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
    fn run(&mut self, command: BatchCommand);

    fn flush(&mut self);
}
