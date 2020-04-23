pub trait BufferInterface<T> {
    fn update_with_range(&mut self, start: usize, end: usize);

    fn copy(&mut self, offset: usize, value: T);

    fn resize(&mut self, size: usize);
}

#[cfg(test)]
pub mod tests {
    use crate::utility::buffer_interface::BufferInterface;

    pub struct MockBuffer {
        pub data: Vec<f32>,
        pub start: usize,
        pub end: usize,
    }

    impl MockBuffer {
        pub fn new(size: usize) -> MockBuffer {
            MockBuffer {
                data: vec![0.0; size],
                start: 0,
                end: 0,
            }
        }

        pub fn get_changes(&self) -> &[f32] {
            &self.data[(self.start as usize)..(self.end as usize)]
        }
    }

    impl BufferInterface<f32> for MockBuffer {
        fn update_with_range(&mut self, start: usize, end: usize) {
            if self.start > start {
                self.start = start;
            }
            if self.end < end {
                self.end = end;
            }
        }

        fn copy(&mut self, offset: usize, value: f32) {
            self.data[offset as usize] = value;
        }

        fn resize(&mut self, size: usize) {
            self.data.resize(size as usize, 0.0f32);
        }
    }
}
