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

    pub fn vertex_buffer(&self) -> &T {
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
        let new_size = self.last() + size - pointer.borrow().size;
        self.size = new_size;

        // Set to update new size
        if pointer.borrow().start < new_size {
            self.change_range
                .resize_and_update(pointer.borrow().start, new_size);
        } else {
            self.change_range.resize(new_size);
        }

        if self.vertices.len() < new_size {
            self.reallocate_vertex_buffer(new_size);
        }

        let old_last = pointer.borrow().last();
        let new_last = old_last + size - pointer.borrow().size;
        if old_size != old_last {
            // If the pointer to be erased is not the last one, it need to re-copy the rest
            let mut copies = vec![0.0f32; old_size - old_last];
            copies.clone_from_slice(&self.vertices[old_last..old_size]);
            self.vertices[new_last..new_size].clone_from_slice(&copies);
        };

        // Find index of target pointer
        let mut after_index = {
            pointer.borrow_mut().size = size;
            let pointer_ptr = &*pointer.borrow() as *const BatchBufferPointer;
            let after_index = self
                .pointers
                .iter()
                .position(move |x| {
                    let x_ptr = &*x.borrow() as *const BatchBufferPointer;
                    std::ptr::eq(x_ptr, pointer_ptr)
                })
                .unwrap();
            after_index + 1
        };

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

    // Call update_with_range when all the copies are done
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

#[cfg(test)]
mod test {
    use crate::core::graphic::batch::batch_buffer::BatchBuffer;
    use crate::core::graphic::batch::batch_buffer_f32::BatchBufferF32Common;
    use crate::core::graphic::hal::vertex_buffer::test::MockVertexBuffer;
    use crate::extension::shared::make_shared;
    use crate::utility::buffer_interface::BufferInterface;
    use crate::utility::test::func::MockFunc;
    use std::ops::{Deref, Range};

    type MockBatchBufferF32 = BatchBufferF32Common<MockVertexBuffer>;

    #[test]
    fn test_copy() {
        let size = 20usize;
        let mock = make_shared(MockFunc::new());
        let vertex_buffer = MockVertexBuffer::new(&mock, &vec![0.0f32; size]);
        let mut batch_buffer = MockBatchBufferF32::new_with_size(
            vertex_buffer,
            |b, v| MockVertexBuffer::new(&b.mock, v),
            size,
        );
        assert_eq!(batch_buffer.vertex_buffer().vertices.borrow().len(), size);
        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().deref(),
            &vec![0.0f32; size]
        );

        let al_0_5 = batch_buffer.allocate(5);
        assert_eq!(al_0_5.borrow().start, 0);
        assert_eq!(al_0_5.borrow().size, 5);

        batch_buffer.copy(0, 1.0f32);
        batch_buffer.update_with_range(0, 1);
        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().deref(),
            &vec![0.0f32; size]
        );

        batch_buffer.flush();
        assert!(
            float_cmp::approx_eq!(
                f32,
                batch_buffer.vertex_buffer().vertices.borrow()[0],
                1.0_f32
            ),
            "v[0] = {}",
            batch_buffer.vertex_buffer().vertices.borrow()[0]
        );
        assert!(
            float_cmp::approx_eq!(
                f32,
                batch_buffer.vertex_buffer().vertices.borrow()[1],
                0.0_f32
            ),
            "v[1] = {}",
            batch_buffer.vertex_buffer().vertices.borrow()[1]
        );
    }

    #[test]
    fn test_allocate() {
        let size = 10usize;
        let mock = make_shared(MockFunc::new());
        let vertex_buffer = MockVertexBuffer::new(&mock, &vec![0.0f32; size]);
        let mut batch_buffer = MockBatchBufferF32::new_with_size(
            vertex_buffer,
            |b, v| MockVertexBuffer::new(&b.mock, v),
            size,
        );
        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().deref(),
            &vec![0.0f32; size]
        );

        let al_0_5 = batch_buffer.allocate(5);
        assert_eq!(al_0_5.borrow().start, 0);
        assert_eq!(al_0_5.borrow().size, 5);
        assert_eq!(batch_buffer.vertex_buffer().vertices.borrow().len(), 10);

        batch_buffer.copy(0, 1.0f32);
        batch_buffer.copy(1, 2.0f32);
        batch_buffer.copy(2, 3.0f32);
        batch_buffer.copy(3, 4.0f32);
        batch_buffer.copy(4, 5.0f32);
        batch_buffer.update_with_range(0, 5);
        batch_buffer.flush();

        let al_5_15 = batch_buffer.allocate(10);
        assert_eq!(al_5_15.borrow().start, 5);
        assert_eq!(al_5_15.borrow().size, 10);
        assert!(batch_buffer.vertex_buffer().vertices.borrow().len() > 10);
        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().to_owned()[0..5],
            [1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32]
        );
    }

    #[test]
    fn test_free() {
        let size = 20usize;
        let mock = make_shared(MockFunc::new());
        let vertex_buffer = MockVertexBuffer::new(&mock, &vec![0.0f32; size]);
        let mut batch_buffer = MockBatchBufferF32::new_with_size(
            vertex_buffer,
            |b, v| MockVertexBuffer::new(&b.mock, v),
            size,
        );
        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().deref(),
            &vec![0.0f32; size]
        );

        let al_0_5 = batch_buffer.allocate(5);
        assert_eq!(al_0_5.borrow().start, 0);
        assert_eq!(al_0_5.borrow().size, 5);

        let al_5_15 = batch_buffer.allocate(10);
        assert_eq!(al_5_15.borrow().start, 5);
        assert_eq!(al_5_15.borrow().size, 10);

        let al_15_27 = batch_buffer.allocate(12);
        assert_eq!(al_15_27.borrow().start, 15);
        assert_eq!(al_15_27.borrow().size, 12);

        let al_27_35 = batch_buffer.allocate(8);
        assert_eq!(al_27_35.borrow().start, 27);
        assert_eq!(al_27_35.borrow().size, 8);

        assert_eq!(batch_buffer.pointers.len(), 4);
        for i in 0..35 {
            batch_buffer.copy(i, i as f32 + 1.0f32);
        }
        batch_buffer.update_with_range(0, 35);
        assert_eq!(
            batch_buffer.change_range.get_range(),
            Some(Range { start: 0, end: 35 })
        );
        batch_buffer.flush();

        batch_buffer.free(&al_5_15);
        assert_eq!(batch_buffer.pointers.len(), 3);
        assert_eq!(
            batch_buffer.change_range.get_range(),
            Some(Range { start: 5, end: 25 })
        );
        batch_buffer.flush();
        assert_eq!(batch_buffer.size, 25);
        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().deref()[0..25],
            [
                1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 16.0f32, 17.0f32, 18.0f32, 19.0f32,
                20.0f32, 21.0f32, 22.0f32, 23.0f32, 24.0f32, 25.0f32, 26.0f32, 27.0f32, 28.0f32,
                29.0f32, 30.0f32, 31.0f32, 32.0f32, 33.0f32, 34.0f32, 35.0f32,
            ]
        );

        batch_buffer.free(&al_27_35);
        assert_eq!(batch_buffer.pointers.len(), 2);
        assert_eq!(batch_buffer.change_range.get_range(), None);
        batch_buffer.flush();
        assert_eq!(batch_buffer.size, 17);
        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().deref()[0..17],
            [
                1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 16.0f32, 17.0f32, 18.0f32, 19.0f32,
                20.0f32, 21.0f32, 22.0f32, 23.0f32, 24.0f32, 25.0f32, 26.0f32, 27.0f32,
            ]
        );

        batch_buffer.free(&al_0_5);
        assert_eq!(batch_buffer.pointers.len(), 1);
        batch_buffer.free(&al_15_27);
        assert_eq!(batch_buffer.pointers.len(), 0);
        assert_eq!(batch_buffer.change_range.get_range(), None);
        batch_buffer.flush();
    }

    #[test]
    pub fn test_reallocate() {
        let size = 20usize;
        let mock = make_shared(MockFunc::new());
        let vertex_buffer = MockVertexBuffer::new(&mock, &vec![0.0f32; size]);
        let mut batch_buffer = MockBatchBufferF32::new_with_size(
            vertex_buffer,
            |b, v| MockVertexBuffer::new(&b.mock, v),
            size,
        );
        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().deref(),
            &vec![0.0f32; size]
        );

        let al_0_5 = batch_buffer.allocate(5);
        let al_5_15 = batch_buffer.allocate(10);
        for i in 0..15 {
            batch_buffer.copy(i, i as f32 + 1.0f32);
        }
        batch_buffer.update_with_range(0, 15);
        batch_buffer.flush();

        batch_buffer.reallocate(&al_0_5, 10);
        batch_buffer.copy(5, 0.0f32);
        batch_buffer.copy(6, 0.0f32);
        batch_buffer.copy(7, 0.0f32);
        batch_buffer.copy(8, 0.0f32);
        batch_buffer.copy(9, 0.0f32);
        assert_eq!(batch_buffer.size(), 20);
        batch_buffer.flush();

        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().deref()[0..20],
            [
                1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
                6.0f32, 7.0f32, 8.0f32, 9.0f32, 10.0f32, 11.0f32, 12.0f32, 13.0f32, 14.0f32,
                15.0f32,
            ]
        );
        assert_eq!(al_5_15.borrow().start, 10);
        assert_eq!(al_5_15.borrow().size, 10);

        batch_buffer.reallocate(&al_5_15, 8);
        batch_buffer.flush();
        assert_eq!(batch_buffer.size(), 18);
        assert_eq!(
            batch_buffer.vertex_buffer().vertices.borrow().deref()[0..18],
            [
                1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
                6.0f32, 7.0f32, 8.0f32, 9.0f32, 10.0f32, 11.0f32, 12.0f32, 13.0f32
            ]
        );
        assert_eq!(al_5_15.borrow().start, 10);
        assert_eq!(al_5_15.borrow().size, 8);
    }
}
