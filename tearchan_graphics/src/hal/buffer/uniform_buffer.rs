use crate::hal::helper::find_memory_type;
use crate::hal::render_bundle::RenderBundleCommon;
use gfx_hal::device::Device;
use gfx_hal::memory::{Properties, Segment};
use gfx_hal::{Backend, MemoryTypeId};
use std::mem::{size_of, ManuallyDrop};

pub struct UniformBufferCommon<B: Backend, T> {
    render_bundle: RenderBundleCommon<B>,
    buffer: ManuallyDrop<B::Buffer>,
    buffer_memory: ManuallyDrop<B::Memory>,
    len: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<B: Backend, T> UniformBufferCommon<B, T> {
    pub fn new(render_bundle: &RenderBundleCommon<B>, data_source: &[T]) -> Self {
        let upload_size = data_source.len() * size_of::<T>();
        let mut buffer = unsafe {
            render_bundle
                .device()
                .create_buffer(upload_size as u64, gfx_hal::buffer::Usage::UNIFORM)
                .unwrap()
        };
        let buffer_req = unsafe { render_bundle.device().get_buffer_requirements(&buffer) };
        let upload_type: MemoryTypeId = find_memory_type(
            render_bundle.memory_types(),
            &buffer_req,
            Properties::CPU_VISIBLE,
        );

        let buffer_memory = unsafe {
            render_bundle
                .device()
                .allocate_memory(upload_type, buffer_req.size)
        }
        .unwrap();

        unsafe {
            let result = render_bundle
                .device()
                .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
                .is_ok();
            debug_assert!(result);

            let mapping = render_bundle
                .device()
                .map_memory(&buffer_memory, Segment::ALL)
                .unwrap();
            std::ptr::copy_nonoverlapping(data_source.as_ptr() as *const u8, mapping, upload_size);
            render_bundle
                .device()
                .flush_mapped_memory_ranges(std::iter::once((&buffer_memory, Segment::ALL)))
                .unwrap();
            render_bundle.device().unmap_memory(&buffer_memory);
        }
        UniformBufferCommon {
            render_bundle: render_bundle.clone(),
            buffer: ManuallyDrop::new(buffer),
            buffer_memory: ManuallyDrop::new(buffer_memory),
            len: upload_size,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn copy_to_buffer(&self, data_source: &[T]) {
        let size = data_source.len() * size_of::<T>();
        unsafe {
            let mapping = self
                .render_bundle
                .device()
                .map_memory(&self.buffer_memory, Segment::ALL)
                .unwrap();
            std::ptr::copy_nonoverlapping(
                data_source.as_ptr() as *const u8,
                mapping,
                size as usize,
            );
            self.render_bundle
                .device()
                .unmap_memory(&self.buffer_memory);
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn buffer(&self) -> &B::Buffer {
        &self.buffer
    }
}

impl<B: Backend, T> Drop for UniformBufferCommon<B, T> {
    fn drop(&mut self) {
        unsafe {
            self.render_bundle
                .device()
                .free_memory(ManuallyDrop::into_inner(std::ptr::read(
                    &self.buffer_memory,
                )));
            self.render_bundle
                .device()
                .destroy_buffer(ManuallyDrop::into_inner(std::ptr::read(&self.buffer)));
        }
    }
}
