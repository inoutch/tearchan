pub trait BufferInterface<T> {
    fn update_with_range(&mut self, start: u32, end: u32);

    fn copy(&mut self, offset: u32, value: T);

    fn resize(&mut self, size: u32);
}

#[cfg(test)]
pub mod tests {
    use crate::utility::buffer_interface::BufferInterface;

    pub struct MockBuffer {
        pub data: Vec<f32>,
        pub start: u32,
        pub end: u32,
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
        fn update_with_range(&mut self, start: u32, end: u32) {
            if self.start > start {
                self.start = start;
            }
            if self.end < end {
                self.end = end;
            }
        }

        fn copy(&mut self, offset: u32, value: f32) {
            self.data[offset as usize] = value;
        }

        fn resize(&mut self, size: u32) {
            self.data.resize(size as usize, 0.0f32);
        }
    }
}
