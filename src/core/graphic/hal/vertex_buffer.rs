use gfx_hal::adapter::MemoryType;
use gfx_hal::device::Device;
use gfx_hal::memory::Segment;
use gfx_hal::{buffer, Backend};
use std::borrow::Borrow;
use std::mem::ManuallyDrop;
use std::option::Option::Some;
use std::rc::{Rc, Weak};
use std::{iter, mem, ptr};

pub trait VertexBufferInterface {
    fn copy_to_buffer(&self, vertices: &[f32], offset: usize, size: usize);
}

pub struct VertexBufferCommon<B: Backend> {
    device: Weak<B::Device>,
    buffer: ManuallyDrop<B::Buffer>,
    buffer_memory: ManuallyDrop<B::Memory>,
    memory_types: Vec<MemoryType>,
    len: usize,
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
            len: vertices.len(),
        }
    }

    pub fn create_vertex_buffer(&self, vertices: &[f32]) -> Option<VertexBufferCommon<B>> {
        if let Some(d) = self.device.upgrade() {
            let (device, buffer, buffer_memory) =
                create_vertex_buffer::<B>(&d, &self.memory_types, vertices);
            Some(VertexBufferCommon {
                device,
                buffer: ManuallyDrop::new(buffer),
                buffer_memory: ManuallyDrop::new(buffer_memory),
                memory_types: self.memory_types.clone(),
                len: vertices.len(),
            })
        } else {
            None
        }
    }

    pub fn borrow_buffer(&self) -> &B::Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<B: Backend> VertexBufferInterface for VertexBufferCommon<B> {
    fn copy_to_buffer(&self, vertices: &[f32], offset: usize, size: usize) {
        if let Some(device) = self.device.upgrade() {
            let binary_offset = offset * std::mem::size_of::<f32>();
            let binary_size = size * std::mem::size_of::<f32>();
            unsafe {
                let memory = self.buffer_memory.borrow();
                let mapping = device
                    .map_memory(
                        memory,
                        Segment {
                            offset: binary_offset as u64,
                            size: Some(binary_size as u64),
                        },
                    )
                    .unwrap();
                ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8, mapping, binary_size);
                device.unmap_memory(memory);
            }
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
    // TODO: Consider alignment
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
                    .contains(gfx_hal::memory::Properties::CPU_VISIBLE)
        })
        .unwrap()
        .into();

    let buffer_memory = unsafe { device.allocate_memory(upload_type, buffer_req.size) }.unwrap();
    unsafe {
        let result = device
            .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
            .is_ok();
        debug_assert!(result);

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
    use crate::core::graphic::hal::vertex_buffer::VertexBufferInterface;
    use crate::extension::shared::{clone_shared, Shared};
    use crate::utility::test::func::MockFunc;
    use std::cell::RefCell;

    pub struct MockVertexBuffer {
        pub mock: Shared<MockFunc>,
        pub vertices: RefCell<Vec<f32>>,
    }

    impl MockVertexBuffer {
        pub fn new(mock: &Shared<MockFunc>, vertices: &[f32]) -> Self {
            MockVertexBuffer {
                mock: clone_shared(mock),
                vertices: RefCell::new(vertices.to_owned()),
            }
        }
    }

    impl VertexBufferInterface for MockVertexBuffer {
        fn copy_to_buffer(&self, vertices: &[f32], offset: usize, size: usize) {
            self.mock.borrow_mut().call(vec![
                "copy_to_buffer".to_string(),
                format!("{:?}", vertices),
                offset.to_string(),
                size.to_string(),
            ]);
            let mut v = self.vertices.borrow_mut();
            v[offset..(offset + size)].clone_from_slice(vertices);
        }
    }
}
