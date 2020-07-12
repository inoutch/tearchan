use crate::core::graphic::hal::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
use gfx_hal::adapter::MemoryType;
use gfx_hal::device::Device;
use gfx_hal::memory::Segment;
use gfx_hal::{buffer, memory, Backend};
use std::mem::ManuallyDrop;
use std::option::Option::Some;
use std::rc::{Rc, Weak};
use std::{iter, mem, ptr};

pub struct VertexBufferCommon<B: Backend> {
    device: Weak<B::Device>,
    buffer: ManuallyDrop<B::Buffer>,
    buffer_memory: ManuallyDrop<B::Memory>,
    memory_types: Vec<MemoryType>,
    size: usize,
}

impl<B: Backend> VertexBufferCommon<B> {
    pub fn new(
        device: &Rc<B::Device>,
        memory_types: &[MemoryType],
        vertices: &[f32],
    ) -> VertexBufferCommon<B> {
        let (device, buffer, buffer_memory) =
            create_vertex_buffer::<B>(device, memory_types, vertices);
        VertexBufferCommon {
            device,
            buffer: ManuallyDrop::new(buffer),
            buffer_memory: ManuallyDrop::new(buffer_memory),
            memory_types: memory_types.to_owned(),
            size: vertices.len(),
        }
    }

    pub fn create_vertex_buffer(&self, vertices: &[f32]) -> Option<VertexBufferCommon<B>> {
        let device = match self.device.upgrade() {
            Some(device) => device,
            None => return None,
        };
        Some(VertexBufferCommon::new(
            &device,
            &self.memory_types,
            vertices,
        ))
    }

    pub fn borrow_buffer(&self) -> &B::Buffer {
        &self.buffer
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl<B: Backend> BufferInterface for VertexBufferCommon<B> {
    type DataType = f32;
    type MappedMemoryType = VertexBufferMemoryMapped;

    fn open(&self, offset: usize, size: usize) -> VertexBufferMemoryMapped {
        let device = self.device.upgrade().unwrap();
        let binary_offset = offset * std::mem::size_of::<f32>();
        let binary_size = size * std::mem::size_of::<f32>();
        let segment = Segment {
            offset: binary_offset as u64,
            size: Some(binary_size as u64),
        };

        let mapping = unsafe { device.map_memory(&self.buffer_memory, segment).unwrap() };
        VertexBufferMemoryMapped {
            mapping,
            binary_size,
        }
    }

    fn close(&self, _mapped_memory: VertexBufferMemoryMapped) {
        let device = self.device.upgrade().unwrap();
        unsafe { device.unmap_memory(&self.buffer_memory) };
    }

    fn size(&self) -> usize {
        self.size
    }
}

pub struct VertexBufferMemoryMapped {
    mapping: *mut u8,
    binary_size: usize,
}

impl BufferMappedMemoryInterface<f32> for VertexBufferMemoryMapped {
    fn copy(&mut self, value: f32, offset: usize) {
        let binary_offset = offset * std::mem::size_of::<f32>();
        debug_assert!(binary_offset < self.binary_size);
        unsafe {
            ptr::copy_nonoverlapping(
                &value as *const f32 as *const u8,
                self.mapping.add(binary_offset),
                std::mem::size_of::<f32>(),
            );
        }
    }
}

impl<B: Backend> Drop for VertexBufferCommon<B> {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                device.destroy_buffer(ManuallyDrop::into_inner(ptr::read(&self.buffer)));
                device.free_memory(ManuallyDrop::into_inner(ptr::read(&self.buffer_memory)));
            }
        }
    }
}

fn create_vertex_buffer<B: Backend>(
    device: &Rc<B::Device>,
    memory_types: &[MemoryType],
    vertices: &[f32],
) -> (Weak<B::Device>, B::Buffer, B::Memory) {
    // TODO: Consider about alignment
    let size = vertices.len() as u64 * mem::size_of::<f32>() as u64;
    let mut buffer = unsafe { device.create_buffer(size, buffer::Usage::VERTEX).unwrap() };
    let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };
    let upload_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, mem_type)| {
            buffer_req.type_mask & (1 << id) as u64 != 0
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
        ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8, mapping, size as usize);
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
    use crate::utility::test::func::MockFunc;

    pub struct MockVertexBuffer {
        pub mock: Shared<MockFunc>,
        pub vertices: Shared<Vec<f32>>,
    }

    impl MockVertexBuffer {
        pub fn new(mock: &Shared<MockFunc>, vertices: &[f32]) -> Self {
            MockVertexBuffer {
                mock: clone_shared(mock),
                vertices: make_shared(vertices.to_owned()),
            }
        }
    }

    impl BufferInterface for MockVertexBuffer {
        type DataType = f32;
        type MappedMemoryType = MockVertexBufferMappedMemory;

        fn open(&self, offset: usize, size: usize) -> MockVertexBufferMappedMemory {
            self.mock
                .borrow_mut()
                .call(vec!["open".to_string(), format!("{}, {}", offset, size)]);
            MockVertexBufferMappedMemory {
                vertices: clone_shared(&self.vertices),
                offset,
                size,
            }
        }

        fn close(&self, _mapped_memory: MockVertexBufferMappedMemory) {
            self.mock.borrow_mut().call(vec!["close".to_string()]);
        }

        fn size(&self) -> usize {
            self.vertices.borrow().len()
        }
    }

    pub struct MockVertexBufferMappedMemory {
        pub vertices: Shared<Vec<f32>>,
        pub offset: usize,
        pub size: usize,
    }

    impl BufferMappedMemoryInterface<f32> for MockVertexBufferMappedMemory {
        fn copy(&mut self, value: f32, offset: usize) {
            assert!(offset < self.size);
            self.vertices.borrow_mut()[offset] = value;
        }
    }
}
