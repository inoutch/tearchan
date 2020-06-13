use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_buffer_pointer::BatchBufferPointer;
use crate::core::graphic::hal::backend::{RendererApi, VertexBuffer};
use crate::core::graphic::hal::vertex_buffer::VertexBufferInterface;
use crate::extension::collection::VecExt;
use crate::extension::shared::{make_shared, Shared};
use crate::math::change_range::ChangeRange;
use crate::utility::buffer_interface::BufferInterface;
use std::ops::Deref;

pub struct BatchBufferF32Common<T: VertexBufferInterface> {
    vertex_buffer: T,
    vertex_buffer_factory: fn(&T, vertices: &Vec<f32>) -> T,
    vertices: Vec<f32>,
    size: usize,
    pointers: Vec<Shared<BatchBufferPointer>>,
    change_range: ChangeRange,
}

impl<T: VertexBufferInterface> BatchBufferF32Common<T> {
    pub fn new_with_size(
        vertex_buffer: T,
        vertex_buffer_factory: fn(&T, vertices: &Vec<f32>) -> T,
        size: usize,
    ) -> BatchBufferF32Common<T> {
        let vertices = vec![0.0f32; size];
        BatchBufferF32Common {
            vertex_buffer,
            vertex_buffer_factory,
            vertices,
            size: 0,
            pointers: vec![],
            change_range: ChangeRange::new(0),
        }
    }

    pub fn borrow_vertex_buffer(&self) -> &T {
        &self.vertex_buffer
    }
}

impl<T: VertexBufferInterface> BatchBufferF32Common<T> {
    fn last(&self) -> usize {
        self.pointers.last().map_or(0, |x| x.borrow().last())
    }

    fn reallocate_vertex_buffer(&mut self, size: usize) {
        self.change_range.resize(size);
        self.change_range.reset();

        let new_size = size * 2;
        self.vertices.resize(new_size, 0.0f32);

        self.size = size;
        let factory = &self.vertex_buffer_factory;
        self.vertex_buffer = factory(&self.vertex_buffer, &self.vertices);
    }
}

impl<T: VertexBufferInterface> BatchBuffer for BatchBufferF32Common<T> {
    fn size(&self) -> usize {
        self.size
    }

    fn allocate(&mut self, size: usize) -> Shared<BatchBufferPointer> {
        let last = self.last();
        self.size += size;
        self.change_range.resize_and_update(last, last + size);

        if last + size > self.vertices.len() {
            self.reallocate_vertex_buffer(last + size);
        }
        let pointer = make_shared(BatchBufferPointer::new(last, size));
        self.pointers.push(Shared::clone(&pointer));
        pointer
    }

    fn reallocate(&mut self, pointer: &Shared<BatchBufferPointer>, size: usize) {
        let old_size = self.size;
        let new_size = self.last() - size - pointer.borrow().size;
        self.size = new_size;

        // Set to update new size
        self.change_range
            .resize_and_update(pointer.borrow().start, new_size);

        if self.vertices.len() < new_size {
            self.reallocate_vertex_buffer(new_size);
        }

        let last = pointer.borrow().last();
        let new_last = last + size - pointer.borrow().size;
        let old_vertices = if old_size == last {
            None
        } else {
            let v = vec![0.0f32; old_size - last];
            self.vertices[last..old_size].clone_from_slice(&v);
            Some(v)
        };

        if let Some(x) = old_vertices {
            let l = x.len();
            let n = l + new_last;
            if n > 0 {
                self.vertices[new_last..n].clone_from_slice(&x[last..(l + last)]);
            }
        }
        pointer.borrow_mut().size = size;
        let pointer_ptr = pointer.borrow().deref() as *const BatchBufferPointer;
        let mut after_index = self
            .pointers
            .iter()
            .position(move |x| {
                let x_ptr = x.borrow().deref() as *const BatchBufferPointer;
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
        let pointer_ptr = pointer.borrow().deref() as *const BatchBufferPointer;
        self.reallocate(pointer, 0);
        self.pointers.remove_item_is(move |x| {
            let x_ptr = x.borrow().deref() as *const BatchBufferPointer;
            std::ptr::eq(x_ptr, pointer_ptr)
        });
    }

    fn sort(&mut self, _sorter: fn(fn(BatchBufferPointer)) -> usize) {
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

impl<T: VertexBufferInterface> BufferInterface<f32> for BatchBufferF32Common<T> {
    fn update_with_range(&mut self, start: usize, end: usize) {
        self.change_range.update(start, end);
    }

    fn copy(&mut self, offset: usize, value: f32) {
        self.vertices[offset] = value;
    }

    fn resize(&mut self, _size: usize) {
        unimplemented!();
    }
}

pub type BatchBufferF32 = BatchBufferF32Common<VertexBuffer>;

impl BatchBufferF32Common<VertexBuffer> {
    pub fn new(api: &RendererApi) -> BatchBufferF32 {
        let size = 256usize;
        let vertices = vec![0.0f32; size];
        BatchBufferF32::new_with_size(
            api.create_vertex_buffer(&vertices),
            |vertex_buffer, vertices| {
                vertex_buffer
                    .create_vertex_buffer(vertices)
                    .expect("device is already dropped")
            },
            1_usize,
        )
    }
}
