use crate::core::graphic::batch::batch_buffer_pointer::BatchBufferPointer;

pub trait BatchBuffer {
    fn size(&self) -> usize;
    fn allocate(&mut self, size: usize) -> BatchBufferPointer;
    fn reallocate(&mut self, pointer: &BatchBufferPointer, size: usize) -> BatchBufferPointer;
    fn free(&mut self, pointer: &BatchBufferPointer);
    fn sort(&mut self, sorter: fn(adder: fn(pointer: BatchBufferPointer)) -> usize);
    fn flush(&mut self);
}

#[cfg(test)]
pub mod tests {
    use crate::core::graphic::batch::batch_buffer::BatchBuffer;
    use crate::core::graphic::batch::batch_buffer_pointer::BatchBufferPointer;
    use crate::extension::shared::Shared;
    use crate::utility::buffer_interface::BufferInterface;
    use crate::utility::test::func::MockFunc;

    pub struct MockBatchBuffer {
        mock_func: Shared<MockFunc>,
    }

    impl MockBatchBuffer {
        pub fn new(mock_func: &Shared<MockFunc>) -> MockBatchBuffer {
            MockBatchBuffer {
                mock_func: Shared::clone(mock_func),
            }
        }
    }

    impl BatchBuffer for MockBatchBuffer {
        fn size(&self) -> usize {
            unimplemented!()
        }

        fn allocate(&mut self, size: usize) -> BatchBufferPointer {
            let mut x = self.mock_func.borrow_mut();
            x.call(vec!["allocate".to_string(), size.to_string()]);
            BatchBufferPointer { start: 0, size: 0 }
        }

        fn reallocate(&mut self, pointer: &BatchBufferPointer, size: usize) -> BatchBufferPointer {
            let mut x = self.mock_func.borrow_mut();
            x.call(vec![
                "reallocate".to_string(),
                format!(
                    "size={}, pointer_start={}, pointer_size={}",
                    size, pointer.start, pointer.start
                ),
            ]);
            BatchBufferPointer { start: 0, size: 0 }
        }

        fn free(&mut self, pointer: &BatchBufferPointer) {
            let mut x = self.mock_func.borrow_mut();
            x.call(vec![
                "reallocate".to_string(),
                format!(
                    "pointer_start={}, pointer_size={}",
                    pointer.start, pointer.start
                ),
            ]);
        }

        fn sort(&mut self, sorter: fn(fn(BatchBufferPointer)) -> usize) {
            unimplemented!()
        }

        fn flush(&mut self) {
            let mut x = self.mock_func.borrow_mut();
            x.call(vec!["flush".to_string()]);
        }
    }

    impl BufferInterface<f32> for MockBatchBuffer {
        fn update_with_range(&mut self, start: usize, end: usize) {
            self.mock_func.borrow_mut().call(vec![
                "update_with_range".to_string(),
                start.to_string(),
                end.to_string(),
            ]);
        }

        fn copy(&mut self, offset: usize, value: f32) {
            self.mock_func.borrow_mut().call(vec![
                "copy".to_string(),
                offset.to_string(),
                value.to_string(),
            ]);
        }

        fn resize(&mut self, size: usize) {
            self.mock_func.borrow_mut().call(vec![
                "resize".to_string(),
                size.to_string()
            ]);
        }
    }
}
