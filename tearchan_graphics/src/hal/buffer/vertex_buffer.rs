use crate::hal::buffer::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
use crate::hal::helper::find_memory_type;
use crate::hal::render_bundle::RenderBundleCommon;
use gfx_hal::device::Device;
use gfx_hal::memory::{Properties, Segment};
use gfx_hal::{buffer, Backend};
use std::mem::ManuallyDrop;
use std::{iter, mem, ptr};
use tearchan_utility::binary::{get_value_from_ptr, set_value_to_ptr};

pub struct VertexBufferCommon<B: Backend> {
    render_bundle: RenderBundleCommon<B>,
    buffer: ManuallyDrop<B::Buffer>,
    buffer_memory: ManuallyDrop<B::Memory>,
    len: usize,
}

impl<B: Backend> VertexBufferCommon<B> {
    pub fn new(render_bundle: &RenderBundleCommon<B>, vertices: &[f32]) -> VertexBufferCommon<B> {
        let (buffer, buffer_memory) = create_vertex_buffer::<B>(render_bundle, vertices);
        VertexBufferCommon {
            render_bundle: render_bundle.clone(),
            buffer: ManuallyDrop::new(buffer),
            buffer_memory: ManuallyDrop::new(buffer_memory),
            len: vertices.len(),
        }
    }

    pub fn create_vertex_buffer(&self, vertices: &[f32]) -> Option<VertexBufferCommon<B>> {
        Some(VertexBufferCommon::new(&self.render_bundle, vertices))
    }

    pub fn get(&self) -> &B::Buffer {
        &self.buffer
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<B: Backend> BufferInterface for VertexBufferCommon<B> {
    type DataType = f32;
    type MappedMemoryType = VertexBufferMappedMemory;

    fn open(&self, offset: usize, size: usize) -> VertexBufferMappedMemory {
        let binary_offset = offset * std::mem::size_of::<f32>();
        let binary_size = size * std::mem::size_of::<f32>();
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
        VertexBufferMappedMemory {
            mapping,
            binary_size,
        }
    }

    fn close(&self, _mapped_memory: VertexBufferMappedMemory) {
        unsafe {
            self.render_bundle
                .device()
                .unmap_memory(&self.buffer_memory)
        };
    }

    fn len(&self) -> usize {
        self.len
    }

    fn clear(&self, offset: usize, size: usize) {
        let mut mapping = self.open(offset, size);
        debug_assert!(offset + size <= self.len);
        for i in 0..size {
            mapping.set(0.0f32, i);
        }
        self.close(mapping);
    }
}

pub struct VertexBufferMappedMemory {
    mapping: *mut u8,
    binary_size: usize,
}

impl BufferMappedMemoryInterface<f32> for VertexBufferMappedMemory {
    fn set(&mut self, value: f32, offset: usize) {
        debug_assert!(offset * std::mem::size_of::<f32>() < self.binary_size);
        unsafe {
            set_value_to_ptr(self.mapping, offset, value);
        }
    }

    fn get(&self, offset: usize) -> f32 {
        debug_assert!(offset * std::mem::size_of::<f32>() < self.binary_size);
        unsafe { get_value_from_ptr(self.mapping, offset, 0.0f32) }
    }
}

impl<B: Backend> Drop for VertexBufferCommon<B> {
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

fn create_vertex_buffer<B: Backend>(
    render_bundle: &RenderBundleCommon<B>,
    vertices: &[f32],
) -> (B::Buffer, B::Memory) {
    // TODO: Consider about alignment
    let size = vertices.len() as u64 * mem::size_of::<f32>() as u64;
    let mut buffer = unsafe {
        render_bundle
            .device()
            .create_buffer(size, buffer::Usage::VERTEX)
            .unwrap()
    };
    let buffer_req = unsafe { render_bundle.device().get_buffer_requirements(&buffer) };
    let upload_type = find_memory_type(
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
        render_bundle
            .device()
            .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
            .unwrap();

        let mapping = render_bundle
            .device()
            .map_memory(&buffer_memory, Segment::ALL)
            .unwrap();
        ptr::copy_nonoverlapping(vertices.as_ptr() as *const u8, mapping, size as usize);

        render_bundle
            .device()
            .flush_mapped_memory_ranges(iter::once((&buffer_memory, Segment::ALL)))
            .unwrap();

        render_bundle.device().unmap_memory(&buffer_memory);
    }
    (buffer, buffer_memory)
}

#[cfg(test)]
pub mod test {
    use crate::hal::buffer::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
    use tearchan_utility::shared::Shared;
    use tearchan_utility::test::mock::MockFunc;

    pub struct MockVertexBuffer {
        pub mock: Shared<MockFunc>,
        pub vertices: Shared<Vec<f32>>,
    }

    impl MockVertexBuffer {
        pub fn new(mock: &Shared<MockFunc>, vertices: &[f32]) -> Self {
            MockVertexBuffer {
                mock: Shared::clone(mock),
                vertices: Shared::new(vertices.to_owned()),
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
                vertices: Shared::clone(&self.vertices),
                offset,
                size,
            }
        }

        fn close(&self, _mapped_memory: MockVertexBufferMappedMemory) {
            self.mock.borrow_mut().call(vec!["close".to_string()]);
        }

        fn len(&self) -> usize {
            self.vertices.borrow().len()
        }

        fn clear(&self, offset: usize, size: usize) {
            for v in &mut self.vertices.borrow_mut()[offset..(offset + size)] {
                *v = 0.0f32;
            }
        }
    }

    pub struct MockVertexBufferMappedMemory {
        pub vertices: Shared<Vec<f32>>,
        pub offset: usize,
        pub size: usize,
    }

    impl BufferMappedMemoryInterface<f32> for MockVertexBufferMappedMemory {
        fn set(&mut self, value: f32, offset: usize) {
            assert!(offset < self.size);
            self.vertices.borrow_mut()[self.offset + offset] = value;
        }

        fn get(&self, offset: usize) -> f32 {
            assert!(offset < self.size);
            self.vertices.borrow()[self.offset + offset]
        }
    }
}
