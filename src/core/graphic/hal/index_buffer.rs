use crate::core::graphic::hal::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
use crate::math::mesh::IndexType;
use crate::utility::binary::{get_value_from_ptr, set_value_to_ptr};
use gfx_hal::adapter::MemoryType;
use gfx_hal::device::Device;
use gfx_hal::memory::Segment;
use gfx_hal::{buffer, memory, Backend};
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};
use std::{iter, mem, ptr};

pub struct IndexBufferCommon<B: Backend> {
    device: Weak<B::Device>,
    buffer: ManuallyDrop<B::Buffer>,
    buffer_memory: ManuallyDrop<B::Memory>,
    memory_types: Vec<MemoryType>,
    size: usize,
}

impl<B: Backend> IndexBufferCommon<B> {
    pub fn new(device: &Rc<B::Device>, memory_types: &[MemoryType], indices: &[IndexType]) -> Self {
        let (device, buffer, buffer_memory) =
            create_index_buffer::<B>(device, memory_types, indices);
        IndexBufferCommon {
            device,
            buffer: ManuallyDrop::new(buffer),
            buffer_memory: ManuallyDrop::new(buffer_memory),
            memory_types: memory_types.to_vec(),
            size: indices.len(),
        }
    }

    pub fn create_index_buffer(&self, indices: &[IndexType]) -> Option<IndexBufferCommon<B>> {
        let device = match self.device.upgrade() {
            None => return None,
            Some(device) => device,
        };

        Some(IndexBufferCommon::new(&device, &self.memory_types, indices))
    }

    pub fn buffer(&self) -> &B::Buffer {
        &self.buffer
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl<B: Backend> BufferInterface for IndexBufferCommon<B> {
    type DataType = IndexType;
    type MappedMemoryType = IndexBufferMappedMemory;

    fn open(&self, offset: usize, size: usize) -> Self::MappedMemoryType {
        let device = self.device.upgrade().unwrap();
        let binary_offset = offset * std::mem::size_of::<IndexType>();
        let binary_size = size * std::mem::size_of::<IndexType>();
        let segment = Segment {
            offset: binary_offset as u64,
            size: Some(binary_size as u64),
        };

        let mapping = unsafe { device.map_memory(&self.buffer_memory, segment).unwrap() };
        IndexBufferMappedMemory {
            mapping,
            binary_size,
        }
    }

    fn close(&self, _mapped_memory: Self::MappedMemoryType) {
        let device = self.device.upgrade().unwrap();
        unsafe { device.unmap_memory(&self.buffer_memory) };
    }

    fn size(&self) -> usize {
        self.size
    }

    fn clear(&self, offset: usize, size: usize) {
        let mut mapping = self.open(offset, size);
        debug_assert!(offset + size <= self.size);
        for i in 0..size {
            mapping.set(0, i);
        }
        self.close(mapping);
    }
}

impl<B: Backend> Drop for IndexBufferCommon<B> {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                device.destroy_buffer(ManuallyDrop::into_inner(ptr::read(&self.buffer)));
                device.free_memory(ManuallyDrop::into_inner(ptr::read(&self.buffer_memory)));
            }
        }
    }
}

pub struct IndexBufferMappedMemory {
    mapping: *mut u8,
    binary_size: usize,
}

impl BufferMappedMemoryInterface<IndexType> for IndexBufferMappedMemory {
    fn set(&mut self, value: IndexType, offset: usize) {
        debug_assert!(offset * std::mem::size_of::<IndexType>() < self.binary_size);
        unsafe {
            set_value_to_ptr(self.mapping, offset, value);
        }
    }

    fn get(&self, offset: usize) -> IndexType {
        debug_assert!(offset * std::mem::size_of::<IndexType>() < self.binary_size);
        unsafe { get_value_from_ptr(self.mapping, offset, 0) }
    }
}

fn create_index_buffer<B: Backend>(
    device: &Rc<B::Device>,
    memory_types: &[MemoryType],
    indices: &[IndexType],
) -> (Weak<B::Device>, B::Buffer, B::Memory) {
    // TODO: Consider about alignment
    let size = indices.len() as u64 * mem::size_of::<IndexType>() as u64;
    let mut buffer = unsafe { device.create_buffer(size, buffer::Usage::INDEX).unwrap() };
    let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };
    let upload_type = memory_types
        .iter()
        .enumerate()
        .position(|(idx, mem_type)| {
            buffer_req.type_mask & (1 << idx) as u32 != 0
                && mem_type
                    .properties
                    .contains(memory::Properties::CPU_VISIBLE)
        })
        .unwrap()
        .into();

    let buffer_memory = unsafe { device.allocate_memory(upload_type, buffer_req.size) }.unwrap();
    unsafe {
        device
            .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
            .unwrap();

        let mapping = device.map_memory(&buffer_memory, Segment::ALL).unwrap();
        ptr::copy_nonoverlapping(indices.as_ptr() as *const u8, mapping, size as usize);
        device
            .flush_mapped_memory_ranges(iter::once((&buffer_memory, Segment::ALL)))
            .unwrap();
        device.unmap_memory(&buffer_memory);
    }
    (Rc::downgrade(device), buffer, buffer_memory)
}

#[cfg(test)]
pub mod test {
    use crate::core::graphic::hal::buffer_interface::{
        BufferInterface, BufferMappedMemoryInterface,
    };
    use crate::extension::shared::{clone_shared, make_shared, Shared};
    use crate::math::mesh::IndexType;
    use crate::utility::test::func::MockFunc;

    pub struct MockIndexBuffer {
        pub mock: Shared<MockFunc>,
        pub indices: Shared<Vec<IndexType>>,
    }

    impl MockIndexBuffer {
        pub fn new(mock: &Shared<MockFunc>, indices: &[IndexType]) -> Self {
            MockIndexBuffer {
                mock: clone_shared(mock),
                indices: make_shared(indices.to_owned()),
            }
        }
    }

    impl BufferInterface for MockIndexBuffer {
        type DataType = IndexType;
        type MappedMemoryType = MockIndexBufferMappedMemory;

        fn open(&self, offset: usize, size: usize) -> MockIndexBufferMappedMemory {
            self.mock
                .borrow_mut()
                .call(vec!["open".to_string(), format!("{}, {}", offset, size)]);
            MockIndexBufferMappedMemory {
                indices: clone_shared(&self.indices),
                offset,
                size,
            }
        }

        fn close(&self, _mapped_memory: MockIndexBufferMappedMemory) {
            self.mock.borrow_mut().call(vec!["close".to_string()]);
        }

        fn size(&self) -> usize {
            self.indices.borrow().len()
        }

        fn clear(&self, offset: usize, size: usize) {
            for v in &mut self.indices.borrow_mut()[offset..(offset + size)] {
                *v = 0;
            }
        }
    }

    pub struct MockIndexBufferMappedMemory {
        pub indices: Shared<Vec<IndexType>>,
        pub offset: usize,
        pub size: usize,
    }

    impl BufferMappedMemoryInterface<IndexType> for MockIndexBufferMappedMemory {
        fn set(&mut self, value: IndexType, offset: usize) {
            assert!(offset < self.size, "{} !< {}", offset, self.size);
            self.indices.borrow_mut()[self.offset + offset] = value;
        }

        fn get(&self, offset: usize) -> u32 {
            assert!(offset < self.size, "{} !< {}", offset, self.size);
            self.indices.borrow()[offset]
        }
    }
}
