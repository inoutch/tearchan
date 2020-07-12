pub trait BufferInterface {
    type DataType;
    type MappedMemoryType: BufferMappedMemoryInterface<Self::DataType>;
    fn open(&self, offset: usize, size: usize)
        -> Self::MappedMemoryType;
    fn close(&self, mapped_memory: Self::MappedMemoryType);
    fn size(&self) -> usize;
}

pub trait BufferMappedMemoryInterface<TDataType> {
    fn copy(&mut self, value: TDataType, offset: usize);
}
