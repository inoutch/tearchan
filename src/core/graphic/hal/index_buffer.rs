use crate::math::mesh::IndexType;
use gfx_hal::adapter::MemoryType;
use gfx_hal::device::Device;
use gfx_hal::memory::Segment;
use gfx_hal::{buffer, memory, Backend};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::{iter, mem, ptr};

pub trait IndexBufferInterface {
    fn copy_to_buffer(&self, vertices: &[IndexType], offset: usize, size: usize);
}

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

impl<B: Backend> IndexBufferInterface for IndexBufferCommon<B> {
    fn copy_to_buffer(&self, vertices: &[u32], offset: usize, size: usize) {
        let device = match self.device.upgrade() {
            Some(device) => device,
            None => return,
        };
        let binary_offset = offset * std::mem::size_of::<IndexType>();
        let binary_size = size * std::mem::size_of::<IndexType>();

        let memory = self.buffer_memory.deref();
        let segment = Segment {
            offset: binary_offset as u64,
            size: Some(binary_size as u64),
        };
        unsafe {
            let mapping = device.map_memory(memory, segment).unwrap();
            ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8, mapping, binary_size);
            device.unmap_memory(memory);
        }
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
            buffer_req.type_mask & (1 << idx) as u64 != 0
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
    use crate::core::graphic::hal::index_buffer::IndexBufferInterface;
    use crate::extension::shared::{clone_shared, Shared};
    use crate::math::mesh::IndexType;
    use crate::utility::test::func::MockFunc;
    use std::cell::RefCell;

    pub struct MockIndexBuffer {
        pub mock: Shared<MockFunc>,
        pub indices: RefCell<Vec<IndexType>>,
    }

    impl MockIndexBuffer {
        pub fn new(mock: &Shared<MockFunc>, indices: &[IndexType]) -> Self {
            MockIndexBuffer {
                mock: clone_shared(mock),
                indices: RefCell::new(indices.to_owned()),
            }
        }
    }

    impl IndexBufferInterface for MockIndexBuffer {
        fn copy_to_buffer(&self, vertices: &[u32], offset: usize, size: usize) {
            self.mock.borrow_mut().call(vec![
                "copy_to_buffer".to_string(),
                format!("{:?}", vertices),
                offset.to_string(),
                size.to_string(),
            ]);

            let mut indices = self.indices.borrow_mut();
            indices[offset..(offset + size)].clone_from_slice(vertices);
        }
    }
}
