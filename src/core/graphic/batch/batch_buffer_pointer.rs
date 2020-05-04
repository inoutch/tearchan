pub struct BatchBufferPointer {
    pub start: usize,
    pub size: usize,
}

impl BatchBufferPointer {
    pub fn last(&self) -> usize {
        self.start + self.size
    }
}
