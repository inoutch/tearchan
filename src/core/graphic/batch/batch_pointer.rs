#[derive(Debug)]
pub struct BatchPointer {
    pub first: usize,
    pub size: usize,
}

impl BatchPointer {
    pub fn new(first: usize, size: usize) -> Self {
        BatchPointer { first, size }
    }

    pub fn last(&self) -> usize {
        self.first + self.size
    }
}

#[cfg(test)]
mod test {
    use crate::core::graphic::batch::batch_pointer::BatchPointer;

    #[test]
    fn test_size() {
        let pointer = BatchPointer::new(10, 30);
        assert_eq!(pointer.last(), 40);
    }
}
