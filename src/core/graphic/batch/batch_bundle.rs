use crate::core::graphic::batch::batch_buffer::BatchBuffer;

pub struct BatchBundle<TBatchBuffer: BatchBuffer> {
    pub stride: u32,
    pub batch_buffer: TBatchBuffer,
}

impl<TBatchBuffer: BatchBuffer> BatchBundle<TBatchBuffer> {}
