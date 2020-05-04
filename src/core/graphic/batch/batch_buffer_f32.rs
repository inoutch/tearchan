use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_buffer_pointer::BatchBufferPointer;
use crate::core::graphic::hal::backend::{FixedApi, FixedVertexBuffer};
use crate::extension::collection::VecExt;
use crate::extension::shared::Shared;
use crate::math::change_range::ChangeRange;
use crate::utility::buffer_interface::BufferInterface;
use std::ops::Deref;

pub struct BatchBufferF32 {
    vertex_buffer: FixedVertexBuffer,
    vertices: Vec<f32>,
    size: usize,
    pointers: Vec<Shared<BatchBufferPointer>>,
    change_range: ChangeRange,
}

impl BatchBufferF32 {
    pub fn new(api: &FixedApi) -> BatchBufferF32 {
        BatchBufferF32::new_with_size(api, 100_000_usize)
    }

    pub fn new_with_size(api: &FixedApi, size: usize) -> BatchBufferF32 {
        let vertices = vec![0.0f32; size];
        let vertex_buffer = api.create_vertex_buffer(&vertices);

        BatchBufferF32 {
            vertex_buffer,
            vertices,
            size: 0,
            pointers: vec![],
            change_range: ChangeRange::new(0),
        }
    }
}

impl BatchBufferF32 {
    fn last(&self) -> usize {
        self.pointers.last().map_or(0, |x| x.last())
    }

    fn reallocate_vertex_buffer(&mut self, size: usize) {
        self.change_range.resize(size);
        self.change_range.update_all();

        let new_size = size * 2;
        self.vertices.resize(new_size, 0.0f32);

        // TODO: reallocate vertex buffer
        self.size = size;
        unimplemented!("reallocate vertex buffer");
    }
}

impl BatchBuffer for BatchBufferF32 {
    fn size(&self) -> usize {
        self.size
    }

    fn allocate(&mut self, size: usize) -> Shared<BatchBufferPointer> {
        let last = self.last();
        self.size += size;

        if last + size > self.vertices.len() {
            self.reallocate_vertex_buffer(last + size);
        }
        let pointer = Shared::new(BatchBufferPointer::new(last, size));
        self.pointers.push(Shared::clone(&pointer));
        pointer
    }

    fn reallocate(&mut self, pointer: &Shared<BatchBufferPointer>, size: usize) {
        let diff = size - pointer.size;
        let old_size = self.size;
        let new_size = self.last() + diff;

        self.change_range.resize_and_update(new_size, pointer.start);

        if self.vertices.len() < new_size {
            self.reallocate_vertex_buffer(new_size);
        }

        let last = pointer.last();
        let new_last = last + diff;
        let old_vertices = if old_size > last {
            None
        } else {
            let v = vec![0.0f32; old_size - last];
            self.vertices[last..old_size].clone_from_slice(&v);
            Some(v)
        };
        if let Some(x) = old_vertices {
            for i in 0..(old_size - last) {
                self.vertices[i + new_last] = x[i + last];
            }
        }
        pointer.borrow_mut().size = size;
        let pointer_ptr = pointer.deref() as *const BatchBufferPointer;
        let mut after_index = self
            .pointers
            .iter()
            .position(move |x| {
                let x_ptr = x.deref() as *const BatchBufferPointer;
                std::ptr::eq(x_ptr, pointer_ptr)
            })
            .unwrap()
            + 1;

        let mut prev_pointer = pointer.borrow_mut();
        while after_index < self.pointers.len() {
            let mut pointer = self.pointers[after_index].borrow_mut();
            pointer.start = prev_pointer.last();
            prev_pointer = pointer;
            after_index += 1;
        }
    }

    fn free(&mut self, pointer: &Shared<BatchBufferPointer>) {
        let pointer_ptr = pointer.deref() as *const BatchBufferPointer;
        self.reallocate(pointer, 0);
        self.pointers.remove_item_is(move |x| {
            let x_ptr = x.deref() as *const BatchBufferPointer;
            std::ptr::eq(x_ptr, pointer_ptr)
        });
    }

    fn sort(&mut self, sorter: fn(fn(BatchBufferPointer)) -> usize) {
        unimplemented!()
    }

    fn flush(&mut self) {
        if let Some(x) = self.change_range.get_range() {
            let size = x.end - x.start;
            if size > 0 {
                self.vertex_buffer
                    .copy_to_buffer(&self.vertices[x.start..x.end], x.start, size);
            }
            self.change_range.reset();
        }
    }
}

impl BufferInterface<f32> for BatchBufferF32 {
    fn update_with_range(&mut self, start: usize, end: usize) {
        self.change_range.update(start, end);
    }

    fn copy(&mut self, offset: usize, value: f32) {
        self.vertices[offset] = value;
    }

    fn resize(&mut self, size: usize) {
        unimplemented!();
    }
}
