use serde::export::fmt::Debug;

#[allow(clippy::needless_range_loop)]
pub trait BufferInterface {
    type DataType: Clone + Debug;
    type MappedMemoryType: BufferMappedMemoryInterface<Self::DataType>;
    fn open(&self, offset: usize, size: usize) -> Self::MappedMemoryType;
    fn close(&self, mapped_memory: Self::MappedMemoryType);
    fn size(&self) -> usize;
    fn clear(&self, offset: usize, size: usize);
}

pub trait BufferMappedMemoryInterface<TDataType> {
    fn set(&mut self, value: TDataType, offset: usize);
    fn get(&self, offset: usize) -> TDataType;
}
