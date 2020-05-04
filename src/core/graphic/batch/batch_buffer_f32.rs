use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_buffer_pointer::BatchBufferPointer;

pub struct BatchBufferF32 {
    
}

impl BatchBuffer for BatchBufferF32 {
    fn size(&self) -> usize {
        unimplemented!()
    }

    fn allocate(&mut self, size: usize) -> BatchBufferPointer {
        unimplemented!()
    }

    fn reallocate(&mut self, pointer: &BatchBufferPointer, size: usize) -> BatchBufferPointer {
        unimplemented!()
    }

    fn free(&mut self, pointer: &BatchBufferPointer) {
        unimplemented!()
    }

    fn sort(&mut self, sorter: fn(fn(BatchBufferPointer)) -> usize) {
        unimplemented!()
    }

    fn flush(&mut self) {
        unimplemented!()
    }
}
