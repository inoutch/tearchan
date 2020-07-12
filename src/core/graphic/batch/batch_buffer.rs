use crate::core::graphic::batch::batch_pointer::BatchPointer;
use crate::core::graphic::hal::buffer_interface::BufferInterface;
use crate::extension::shared::{clone_shared, make_shared, Shared};
use crate::math::change_range::ChangeRange;
use std::collections::{HashMap, VecDeque};
use std::ops::Deref;

pub struct BatchBuffer<TBuffer: BufferInterface> {
    buffer: TBuffer,
    buffer_factory: fn(&TBuffer, usize) -> TBuffer,
    pointer_indices: HashMap<*const BatchPointer, usize>,
    pointers: VecDeque<Shared<BatchPointer>>,
    change_pointer_range: ChangeRange,
}

impl<TBuffer: BufferInterface> BatchBuffer<TBuffer> {
    pub fn new(buffer: TBuffer, buffer_factory: fn(buffer: &TBuffer, usize) -> TBuffer) -> Self {
        BatchBuffer {
            buffer,
            buffer_factory,
            pointers: VecDeque::new(),
            pointer_indices: HashMap::new(),
            change_pointer_range: ChangeRange::new(0),
        }
    }

    pub fn allocate(&mut self, size: usize) -> Shared<BatchPointer> {
        let first = self.last();
        if first + size > self.buffer.size() {
            self.reallocate_buffer(first + size);
        }

        let index = self.pointers.len();
        let (key, ptr) = {
            let ptr = make_shared(BatchPointer::new(index, first, size));
            let key = get_batch_pointer_key(&ptr);
            (key, ptr)
        };

        self.pointers.push_back(clone_shared(&ptr));
        self.pointer_indices.insert(key, index);

        self.change_pointer_range.resize(self.pointers.len());
        ptr
    }

    pub fn reallocate(&mut self, ptr: &Shared<BatchPointer>, size: usize) -> usize {
        self.reallocate_or_free(ptr, size, false)
    }

    pub fn free(&mut self, pointer: &Shared<BatchPointer>) {
        let index = self.reallocate_or_free(pointer, 0, true);
        self.pointers.remove(index);
        self.pointer_indices.remove(&get_batch_pointer_key(pointer));
        self.change_pointer_range.reset();
        self.change_pointer_range
            .update(index, self.pointer_indices.len());
    }

    pub fn buffer(&self) -> &TBuffer {
        &self.buffer
    }

    pub fn size(&self) -> usize {
        self.buffer.size()
    }

    pub fn last(&self) -> usize {
        if self.pointers.is_empty() {
            0usize
        } else {
            self.pointers[self.pointers.len() - 1].borrow().last()
        }
    }

    pub fn change_pointer_range(&self) -> &ChangeRange {
        &self.change_pointer_range
    }

    pub fn flush(&mut self) {
        self.change_pointer_range.reset();
    }

    fn reallocate_buffer(&mut self, size: usize) {
        let new_size = size * 2;
        let factory = &self.buffer_factory;
        self.buffer = factory(&self.buffer, new_size);
    }

    fn reallocate_or_free(&mut self, ptr: &Shared<BatchPointer>, size: usize, free: bool) -> usize {
        let index = self.pointer_indices[&get_batch_pointer_key(ptr)];
        let new_size = self.last() + size - ptr.borrow().size;

        // Set to update new size
        if ptr.borrow().size < size {
            self.change_pointer_range
                .update(index, self.change_pointer_range.size);
        } else {
            self.change_pointer_range
                .update(index + 1, self.change_pointer_range.size);
        }

        if self.buffer.size() < new_size {
            self.reallocate_buffer(new_size);
        }

        ptr.borrow_mut().size = size;

        let mut after_index = index + 1;
        let mut prev_pointer = ptr.borrow_mut();
        while after_index < self.pointers.len() {
            let mut pointer = {
                let pointer = &self.pointers[after_index];
                if free {
                    let next_index = after_index - 1;
                    pointer.borrow_mut().index = next_index;
                    self.pointer_indices
                        .insert(get_batch_pointer_key(pointer), next_index);
                }
                pointer.borrow_mut()
            };
            pointer.first = prev_pointer.last();
            prev_pointer = pointer;
            after_index += 1;
        }
        index
    }
}

#[inline]
fn get_batch_pointer_key(ptr: &Shared<BatchPointer>) -> *const BatchPointer {
    ptr.borrow().deref() as *const BatchPointer
}

#[cfg(test)]
mod test {
    use crate::core::graphic::batch::batch_buffer::{get_batch_pointer_key, BatchBuffer};
    use crate::core::graphic::hal::vertex_buffer::test::MockVertexBuffer;
    use crate::extension::shared::make_shared;
    use crate::utility::test::func::MockFunc;
    use std::ops::Range;

    #[test]
    fn test_batch_buffer() {
        let mock = make_shared(MockFunc::new());
        let mock_vertex_buffer = MockVertexBuffer::new(&mock, vec![0.0f32; 32].as_slice());
        let mut batch_buffer = BatchBuffer::new(mock_vertex_buffer, |buffer, size| {
            let mut vertices = buffer.vertices.borrow().clone();
            vertices.resize(size, 0.0f32);
            MockVertexBuffer::new(&buffer.mock, &vertices)
        });

        let p1 = batch_buffer.allocate(30);
        assert_eq!(p1.borrow().first, 0);
        assert_eq!(p1.borrow().last(), 30);
        assert_eq!(p1.borrow().index, 0);
        assert_eq!(batch_buffer.pointers.len(), 1);
        assert_eq!(batch_buffer.pointer_indices.len(), 1);

        let p2 = batch_buffer.allocate(50);
        assert_eq!(p2.borrow().first, 30);
        assert_eq!(p2.borrow().last(), 80);
        assert_eq!(p2.borrow().index, 1);
        assert_eq!(batch_buffer.pointers.len(), 2);
        assert_eq!(batch_buffer.pointer_indices.len(), 2);

        let p3 = batch_buffer.allocate(42);
        assert_eq!(p3.borrow().first, 80);
        assert_eq!(p3.borrow().last(), 122);
        assert_eq!(p3.borrow().index, 2);
        assert_eq!(batch_buffer.pointers.len(), 3);
        assert_eq!(batch_buffer.pointer_indices.len(), 3);

        assert_eq!(batch_buffer.size(), 160); // (30 + 50) * 2
        assert_eq!(batch_buffer.last(), 122);
        assert_eq!(
            batch_buffer.change_pointer_range().get_range(),
            Some(Range { start: 0, end: 3 })
        );

        batch_buffer.flush();
        assert_eq!(batch_buffer.change_pointer_range().get_range(), None);

        batch_buffer.reallocate(&p1, 40);
        assert_eq!(p1.borrow().first, 0);
        assert_eq!(p1.borrow().last(), 40);
        assert_eq!(p1.borrow().index, 0);
        assert_eq!(p2.borrow().first, 40);
        assert_eq!(p2.borrow().last(), 90);
        assert_eq!(p2.borrow().index, 1);
        assert_eq!(p3.borrow().first, 90);
        assert_eq!(p3.borrow().last(), 132);
        assert_eq!(p3.borrow().index, 2);

        assert_eq!(batch_buffer.size(), 160); // (30 + 50) * 2
        assert_eq!(batch_buffer.last(), 132);

        assert_eq!(
            batch_buffer.change_pointer_range().get_range(),
            Some(Range { start: 0, end: 3 })
        );
        batch_buffer.flush();

        batch_buffer.free(&p2);
        assert_eq!(p1.borrow().first, 0);
        assert_eq!(p1.borrow().last(), 40);
        assert_eq!(p1.borrow().index, 0);
        assert_eq!(p3.borrow().first, 40);
        assert_eq!(p3.borrow().last(), 82);
        assert_eq!(p3.borrow().index, 1);
        assert_eq!(batch_buffer.size(), 160);
        assert_eq!(batch_buffer.last(), 82);
        assert_eq!(batch_buffer.pointers.len(), 2);
        assert_eq!(batch_buffer.pointer_indices.len(), 2);
        assert_eq!(batch_buffer.pointer_indices[&get_batch_pointer_key(&p3)], 1);

        assert_eq!(
            batch_buffer.change_pointer_range().get_range(),
            Some(Range { start: 1, end: 2 })
        );
    }
}
