use std::cmp::Ordering;

#[allow(clippy::needless_range_loop)]
pub trait BufferInterface {
    type DataType: Clone;
    type MappedMemoryType: BufferMappedMemoryInterface<Self::DataType>;
    fn open(&self, offset: usize, size: usize) -> Self::MappedMemoryType;
    fn close(&self, mapped_memory: Self::MappedMemoryType);
    fn size(&self) -> usize;
    fn clear(&self, offset: usize, size: usize);
    fn copy_to(&self, from: usize, to: usize, size: usize) {
        let (start, end) = match from.cmp(&to) {
            Ordering::Less => (from, to + size),
            Ordering::Greater => (to, from + size),
            Ordering::Equal => return,
        };
        let mut mapping = self.open(start, end - size);
        let mut buffers: Vec<Self::DataType> = Vec::with_capacity(size);

        // Copy from
        for i in 0..size {
            buffers.push(mapping.get(i + from));
        }

        // Copy to
        for i in 0..size {
            mapping.set(buffers[i].clone(), i + to);
        }
    }
}

pub trait BufferMappedMemoryInterface<TDataType> {
    fn set(&mut self, value: TDataType, offset: usize);
    fn get(&self, offset: usize) -> TDataType;
}
