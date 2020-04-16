pub trait BufferInterface<T> {
    fn update_with_range(&mut self, start: u16, end: u16);

    fn copy(&mut self, offset: u16, value: T);

    fn resize(&mut self, size: u16);
}

#[cfg(test)]
pub mod tests {
    use crate::utility::buffer_interface::BufferInterface;

    pub struct MockBuffer {
        data: Vec<f32>,
        start: u16,
        end: u16,
    }

    impl MockBuffer {
        pub fn new(size: usize) -> MockBuffer {
            MockBuffer {
                data: vec![0.0; size],
                start: 0,
                end: 0,
            }
        }
    }

    impl BufferInterface<f32> for MockBuffer {
        fn update_with_range(&mut self, start: u16, end: u16) {
            if self.start > start {
                self.start = start;
            }
            if self.end > end {
                self.end = end;
            }
        }

        fn copy(&mut self, offset: u16, value: f32) {
            self.data[offset as usize] = value;
        }

        fn resize(&mut self, size: u16) {
            self.data.resize(size as usize, 0.0f32);
        }
    }
}
