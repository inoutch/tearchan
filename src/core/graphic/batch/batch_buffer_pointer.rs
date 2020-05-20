pub struct BatchBufferPointer {
    pub start: usize,
    pub size: usize,
}

impl BatchBufferPointer {
    pub fn new(start: usize, size: usize) -> Self {
        BatchBufferPointer { start, size }
    }

    pub fn last(&self) -> usize {
        self.start + self.size
    }
}
