use gfx_hal::adapter::MemoryType;
use gfx_hal::device::Device;
use gfx_hal::memory::Segment;
use gfx_hal::{buffer, Backend};
use std::borrow::Borrow;
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};
use std::{iter, mem, ptr};

pub struct VertexBuffer<B: Backend> {
    device: Weak<B::Device>,
    buffer: ManuallyDrop<B::Buffer>,
    buffer_memory: ManuallyDrop<B::Memory>,
}

impl<B: Backend> VertexBuffer<B> {
    pub fn new(
        device: &Rc<B::Device>,
        memory_types: &[MemoryType],
        vertices: &[f32],
    ) -> VertexBuffer<B> {
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

        let buffer_memory =
            unsafe { device.allocate_memory(upload_type, buffer_req.size) }.unwrap();
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

        VertexBuffer {
            device: Rc::downgrade(device),
            buffer: ManuallyDrop::new(buffer),
            buffer_memory: ManuallyDrop::new(buffer_memory),
        }
    }

    pub fn copy_to_buffer(&self, vertices: &[f32], offset: usize, size: usize) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                let memory = self.buffer_memory.borrow();
                let mapping = device
                    .map_memory(
                        memory,
                        Segment {
                            offset: offset as u64,
                            size: Some(size as u64),
                        },
                    )
                    .unwrap();
                ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8, mapping, size as usize);
                device.unmap_memory(memory);
            }
        }
    }

    pub fn borrow_buffer(&self) -> &B::Buffer {
        &self.buffer
    }
}

impl<B: Backend> Drop for VertexBuffer<B> {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                device.destroy_buffer(ManuallyDrop::into_inner(ptr::read(&self.buffer)));
                device.free_memory(ManuallyDrop::into_inner(ptr::read(&self.buffer_memory)));
            }
        }
    }
}
