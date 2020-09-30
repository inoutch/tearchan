use crate::hal::buffer::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
use crate::hal::helper::find_memory_type;
use crate::hal::render_bundle::RenderBundleCommon;
use gfx_hal::adapter::MemoryType;
use gfx_hal::device::Device;
use gfx_hal::memory::{Properties, Segment};
use gfx_hal::{buffer, Backend};
use std::mem::ManuallyDrop;
use std::{iter, mem, ptr};
use tearchan_utility::binary::{get_value_from_ptr, set_value_to_ptr};
use tearchan_utility::mesh::IndexType;

pub struct IndexBufferCommon<B: Backend> {
    render_bundle: RenderBundleCommon<B>,
    buffer: ManuallyDrop<B::Buffer>,
    buffer_memory: ManuallyDrop<B::Memory>,
    len: usize,
}

impl<B: Backend> IndexBufferCommon<B> {
    pub fn new(render_bundle: &RenderBundleCommon<B>, indices: &[IndexType]) -> Self {
        let (buffer, buffer_memory) = create_index_buffer::<B>(
            render_bundle.device(),
            render_bundle.memory_types(),
            indices,
        );
        IndexBufferCommon {
            render_bundle: render_bundle.clone(),
            buffer: ManuallyDrop::new(buffer),
            buffer_memory: ManuallyDrop::new(buffer_memory),
            len: indices.len(),
        }
    }

    pub fn create_index_buffer(&self, indices: &[IndexType]) -> Option<IndexBufferCommon<B>> {
        Some(IndexBufferCommon::new(&self.render_bundle, indices))
    }

    pub fn get(&self) -> &B::Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<B: Backend> BufferInterface for IndexBufferCommon<B> {
    type DataType = IndexType;
    type MappedMemoryType = IndexBufferMappedMemory;

    fn open(&self, offset: usize, size: usize) -> Self::MappedMemoryType {
        let binary_offset = offset * std::mem::size_of::<IndexType>();
        let binary_size = size * std::mem::size_of::<IndexType>();
        let segment = Segment {
            offset: binary_offset as u64,
            size: Some(binary_size as u64),
        };

        let mapping = unsafe {
            self.render_bundle
                .device()
                .map_memory(&self.buffer_memory, segment)
                .unwrap()
        };
        IndexBufferMappedMemory {
            mapping,
            binary_size,
        }
    }

    fn close(&self, _mapped_memory: Self::MappedMemoryType) {
        unsafe {
            self.render_bundle
                .device()
                .unmap_memory(&self.buffer_memory)
        };
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn clear(&self, offset: usize, size: usize) {
        let mut mapping = self.open(offset, size);
        debug_assert!(offset + size <= self.len);
        for i in 0..size {
            mapping.set(0, i);
        }
        self.close(mapping);
    }
}

impl<B: Backend> Drop for IndexBufferCommon<B> {
    fn drop(&mut self) {
        unsafe {
            self.render_bundle
                .device()
                .destroy_buffer(ManuallyDrop::into_inner(ptr::read(&self.buffer)));
            self.render_bundle
                .device()
                .free_memory(ManuallyDrop::into_inner(ptr::read(&self.buffer_memory)));
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
    device: &B::Device,
    memory_types: &[MemoryType],
    indices: &[IndexType],
) -> (B::Buffer, B::Memory) {
    // TODO: Consider about alignment
    let size = indices.len() as u64 * mem::size_of::<IndexType>() as u64;
    let mut buffer = unsafe { device.create_buffer(size, buffer::Usage::INDEX).unwrap() };
    let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };
    let upload_type = find_memory_type(memory_types, &buffer_req, Properties::CPU_VISIBLE);

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
    (buffer, buffer_memory)
}

#[cfg(test)]
pub mod test {
    use crate::hal::buffer::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
    use tearchan_utility::mesh::IndexType;
    use tearchan_utility::shared::Shared;
    use tearchan_utility::test::mock::MockFunc;

    pub struct MockIndexBuffer {
        pub mock: Shared<MockFunc>,
        pub indices: Shared<Vec<IndexType>>,
    }

    impl MockIndexBuffer {
        pub fn new(mock: &Shared<MockFunc>, indices: &[IndexType]) -> Self {
            MockIndexBuffer {
                mock: Shared::clone(mock),
                indices: Shared::new(indices.to_owned()),
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
                indices: Shared::clone(&self.indices),
                offset,
                size,
            }
        }

        fn close(&self, _mapped_memory: MockIndexBufferMappedMemory) {
            self.mock.borrow_mut().call(vec!["close".to_string()]);
        }

        fn len(&self) -> usize {
            self.indices.borrow().len()
        }

        fn is_empty(&self) -> bool {
            self.indices.borrow().is_empty()
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
