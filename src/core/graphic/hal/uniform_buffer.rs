use gfx_hal::adapter::MemoryType;
use gfx_hal::device::Device;
use gfx_hal::memory::Segment;
use gfx_hal::{Backend, MemoryTypeId};
use std::borrow::Borrow;
use std::mem::{size_of, ManuallyDrop};
use std::rc::{Rc, Weak};

pub struct UniformBuffer<B: Backend, T> {
    device: Weak<B::Device>,
    buffer: ManuallyDrop<B::Buffer>,
    buffer_memory: ManuallyDrop<B::Memory>,
    size: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<B: Backend, T> UniformBuffer<B, T> {
    pub fn new(device: &Rc<B::Device>, memory_types: &[MemoryType], data_source: &[T]) -> Self {
        let upload_size = data_source.len() * size_of::<T>();
        let mut buffer = unsafe {
            device
                .create_buffer(upload_size as u64, gfx_hal::buffer::Usage::UNIFORM)
                .unwrap()
        };
        let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };
        let upload_type: MemoryTypeId = memory_types
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
            std::ptr::copy_nonoverlapping(data_source.as_ptr() as *const u8, mapping, upload_size);
            device
                .flush_mapped_memory_ranges(std::iter::once((&buffer_memory, Segment::ALL)))
                .unwrap();
            device.unmap_memory(&buffer_memory);
        }
        UniformBuffer {
            device: Rc::downgrade(device),
            buffer: ManuallyDrop::new(buffer),
            buffer_memory: ManuallyDrop::new(buffer_memory),
            size: upload_size,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn copy_to_buffer(&self, data_source: &[T]) {
        if let Some(device) = self.device.upgrade() {
            let size = data_source.len() * size_of::<T>();
            unsafe {
                let memory = self.buffer_memory.borrow();
                let mapping = device.map_memory(memory, Segment::ALL).unwrap();
                std::ptr::copy_nonoverlapping(
                    data_source.as_ptr() as *const u8,
                    mapping,
                    size as usize,
                );
                device.unmap_memory(memory);
            }
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn borrow_buffer(&self) -> &B::Buffer {
        &self.buffer
    }
}

impl<B: Backend, T> Drop for UniformBuffer<B, T> {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                device.free_memory(ManuallyDrop::into_inner(std::ptr::read(
                    &self.buffer_memory,
                )));
                device.destroy_buffer(ManuallyDrop::into_inner(std::ptr::read(&self.buffer)));
            }
        }
    }
}
