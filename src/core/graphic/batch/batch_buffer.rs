use crate::core::graphic::batch::batch_pointer::BatchPointer;
use crate::core::graphic::hal::buffer_interface::BufferInterface;
use crate::extension::shared::{clone_shared, make_shared, Shared};
use crate::utility::btree::DuplicatableBTreeMap;
use std::collections::HashMap;
use std::ops::Deref;

pub struct BatchBuffer<TBuffer: BufferInterface> {
    buffer: TBuffer,
    buffer_factory: fn(&TBuffer, usize) -> TBuffer,
    pointers: HashMap<*const BatchPointer, Shared<BatchPointer>>,
    last: usize,
    pending_pointers: DuplicatableBTreeMap<usize, Shared<BatchPointer>>,
    fragmentation_size: usize,
}

impl<TBuffer: BufferInterface> BatchBuffer<TBuffer> {
    pub fn new(buffer: TBuffer, buffer_factory: fn(buffer: &TBuffer, usize) -> TBuffer) -> Self {
        BatchBuffer {
            buffer,
            buffer_factory,
            pointers: HashMap::new(),
            last: 0,
            pending_pointers: DuplicatableBTreeMap::new(),
            fragmentation_size: 0,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Shared<BatchPointer> {
        // Search from pending_pointers
        if let Some(ptr) = match self.pending_pointers.range_mut(size..).next() {
            Some((_, pointers)) => pointers.pop_back(),
            None => None,
        } {
            // Reuse the memory if there is more free space than the desired size
            let (key, ptr) = {
                let key = get_batch_pointer_key(&ptr);
                (key, ptr)
            };
            self.pointers.insert(key, clone_shared(&ptr));
            self.fragmentation_size -= ptr.borrow().size; // Note that reduce will increase the fragment size

            if ptr.borrow().size != size {
                // Reducing unnecessary memory size
                self.reduce_pointer(&ptr, size);
            }
            ptr
        } else {
            // Allocate new memory space
            let (key, ptr) = self.allocate_new_pointer(size);
            self.pointers.insert(key, clone_shared(&ptr));
            ptr
        }
    }

    pub fn reallocate(&mut self, pointer: &Shared<BatchPointer>, size: usize) {
        let pointer_size = pointer.borrow().size;
        match pointer_size {
            d if d > size => self.reduce_pointer(pointer, size),
            d if d < size => {
                let (old_first, old_size) = {
                    let borrow = pointer.borrow();
                    (borrow.first, borrow.size)
                };
                let mut old_ptr = pointer.borrow_mut();
                let new_ptr = self.allocate(size);
                // Swap values
                {
                    let mut new_ptr_borrow_mut = new_ptr.borrow_mut();

                    old_ptr.first = new_ptr_borrow_mut.first;
                    old_ptr.size = new_ptr_borrow_mut.size;
                    new_ptr_borrow_mut.first = old_first;
                    new_ptr_borrow_mut.size = old_size;

                    self.buffer.clear(old_first, old_size);
                    self.fragmentation_size += old_size;
                }
                // Swap registers
                self.pointers.remove(&get_batch_pointer_key(&new_ptr));
                self.pending_pointers.push_back(old_size, new_ptr);
            }
            _ => {}
        }
    }

    pub fn free(&mut self, pointer: &Shared<BatchPointer>) {
        self.pending_pointers
            .push_back(pointer.borrow().size, clone_shared(pointer));
        self.pointers.remove(&get_batch_pointer_key(pointer));
        self.fragmentation_size += pointer.borrow().size;
        self.buffer
            .clear(pointer.borrow().first, pointer.borrow().size);
    }

    pub fn buffer(&self) -> &TBuffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut TBuffer {
        &mut self.buffer
    }

    pub fn size(&self) -> usize {
        self.buffer.size()
    }

    pub fn last(&self) -> usize {
        self.last
    }

    pub fn fragmentation_size(&self) -> usize {
        self.fragmentation_size
    }

    fn reallocate_buffer(&mut self, size: usize) {
        let new_size = size * 2;
        let factory = &self.buffer_factory;
        self.buffer = factory(&self.buffer, new_size);
    }

    #[allow(clippy::needless_range_loop)]
    pub fn defragmentation(&mut self) {
        let mut cloned_pointers: Vec<_> = self
            .pointers
            .values()
            .map(|pointer| clone_shared(pointer))
            .collect();
        cloned_pointers.sort_unstable_by(|a, b| a.borrow().first.cmp(&b.borrow().first));

        let mut first: usize = 0;
        for pointer in cloned_pointers {
            pointer.borrow_mut().first = first;
            first += pointer.borrow().size;
        }

        self.last = first;
        self.pending_pointers.clear();
        self.fragmentation_size = 0;
    }

    fn allocate_new_pointer(&mut self, size: usize) -> (*const BatchPointer, Shared<BatchPointer>) {
        let first = self.last;
        if first + size > self.buffer.size() {
            self.reallocate_buffer(first + size);
        }

        let (key, ptr) = {
            let ptr = make_shared(BatchPointer::new(first, size));
            let key = get_batch_pointer_key(&ptr);
            (key, ptr)
        };
        self.last += size;
        (key, ptr)
    }

    fn reduce_pointer(&mut self, pointer: &Shared<BatchPointer>, size: usize) {
        let mut pointer_mut = pointer.borrow_mut();
        if pointer_mut.last() != self.last {
            let r_first = pointer_mut.first + size;
            let r_size = pointer_mut.size - size;
            let r_ptr = make_shared(BatchPointer::new(r_first, r_size));
            self.pending_pointers.push_back(r_size, r_ptr);

            self.buffer.clear(r_first, r_size);
            self.fragmentation_size += r_size;
        } else {
            self.last = pointer_mut.first + size;
        }

        pointer_mut.size = size;
    }
}

#[inline]
fn get_batch_pointer_key(ptr: &Shared<BatchPointer>) -> *const BatchPointer {
    ptr.borrow().deref() as *const BatchPointer
}

#[cfg(test)]
mod test {
    use crate::core::graphic::batch::batch_buffer::BatchBuffer;
    use crate::core::graphic::hal::buffer_interface::{
        BufferInterface, BufferMappedMemoryInterface,
    };
    use crate::core::graphic::hal::index_buffer::test::MockIndexBuffer;
    use crate::core::graphic::hal::vertex_buffer::test::MockVertexBuffer;
    use crate::extension::shared::make_shared;
    use crate::utility::test::func::MockFunc;

    #[test]
    fn test_batch_buffer_memory_allocation() {
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
        assert_eq!(batch_buffer.pointers.len(), 1);

        let p2 = batch_buffer.allocate(50);
        assert_eq!(p2.borrow().first, 30);
        assert_eq!(p2.borrow().last(), 80);
        assert_eq!(batch_buffer.pointers.len(), 2);

        let p3 = batch_buffer.allocate(42);
        assert_eq!(p3.borrow().first, 80);
        assert_eq!(p3.borrow().last(), 122);
        assert_eq!(batch_buffer.pointers.len(), 3);

        assert_eq!(batch_buffer.size(), 160); // (30 + 50) * 2
        assert_eq!(batch_buffer.last(), 122);

        batch_buffer.reallocate(&p1, 40);
        assert_eq!(p1.borrow().first, 122);
        assert_eq!(p1.borrow().last(), 162);
        assert_eq!(p2.borrow().first, 30);
        assert_eq!(p2.borrow().last(), 80);
        assert_eq!(p3.borrow().first, 80);
        assert_eq!(p3.borrow().last(), 122);

        assert_eq!(batch_buffer.size(), 324); // (30 + 50) * 2 => 162 * 2
        assert_eq!(batch_buffer.last(), 162);
        assert_eq!(batch_buffer.pending_pointers.len(), 1);

        batch_buffer.free(&p2);
        assert_eq!(p1.borrow().first, 122);
        assert_eq!(p1.borrow().last(), 162);
        assert_eq!(p3.borrow().first, 80);
        assert_eq!(p3.borrow().last(), 122);
        assert_eq!(batch_buffer.size(), 324);
        assert_eq!(batch_buffer.last(), 162);
        assert_eq!(batch_buffer.pointers.len(), 2);
        assert_eq!(batch_buffer.pending_pointers.len(), 2);

        batch_buffer.reallocate(&p3, 40);
        assert_eq!(p3.borrow().first, 80);
        assert_eq!(p3.borrow().last(), 120);
        assert_eq!(batch_buffer.last(), 162);
        assert_eq!(batch_buffer.pointers.len(), 2);
        assert_eq!(batch_buffer.pending_pointers.len(), 3);

        batch_buffer.reallocate(&p1, 30);
        assert_eq!(p1.borrow().first, 122);
        assert_eq!(p1.borrow().last(), 152);
        assert_eq!(batch_buffer.last(), 152);
        assert_eq!(batch_buffer.pointers.len(), 2);
        assert_eq!(batch_buffer.pending_pointers.len(), 3);

        batch_buffer.defragmentation();
        assert_eq!(batch_buffer.last(), 70);
        assert_eq!(batch_buffer.pending_pointers.len(), 0);
        assert_eq!(batch_buffer.fragmentation_size, 0);
    }

    #[test]
    fn test_batch_buffer_values() {
        let mock = make_shared(MockFunc::new());
        let mock_vertex_buffer = MockIndexBuffer::new(&mock, vec![0u32; 32].as_slice());
        let mut batch_buffer = BatchBuffer::new(mock_vertex_buffer, |buffer, size| {
            let mut indices = buffer.indices.borrow().clone();
            indices.resize(size, 0u32);
            MockIndexBuffer::new(&buffer.mock, &indices)
        });

        let p1 = batch_buffer.allocate(8);
        {
            let mut mapping = batch_buffer
                .buffer()
                .open(p1.borrow().first, p1.borrow().size);
            for i in 0..8 {
                mapping.set(i, i as usize);
            }
            batch_buffer.buffer().close(mapping);
        }
        assert_eq!(
            &batch_buffer.buffer.indices.borrow()[0..batch_buffer.last()],
            &[0u32, 1u32, 2u32, 3u32, 4u32, 5u32, 6u32, 7u32]
        );
        assert_eq!(batch_buffer.fragmentation_size, 0);

        let p2 = batch_buffer.allocate(3);
        {
            let mut mapping = batch_buffer
                .buffer()
                .open(p2.borrow().first, p2.borrow().size);
            for i in 0..3 {
                mapping.set(i, i as usize);
            }
            batch_buffer.buffer().close(mapping);
        }
        assert_eq!(
            &batch_buffer.buffer.indices.borrow()[0..batch_buffer.last()],
            &[0u32, 1u32, 2u32, 3u32, 4u32, 5u32, 6u32, 7u32, 0u32, 1u32, 2u32]
        );
        assert_eq!(batch_buffer.fragmentation_size, 0);

        let p3 = batch_buffer.allocate(7);
        {
            let mut mapping = batch_buffer
                .buffer()
                .open(p3.borrow().first, p3.borrow().size);
            for i in 0..7 {
                mapping.set(i, i as usize);
            }
            batch_buffer.buffer().close(mapping);
        }
        assert_eq!(
            &batch_buffer.buffer.indices.borrow()[0..batch_buffer.last()],
            &[
                0u32, 1u32, 2u32, 3u32, 4u32, 5u32, 6u32, 7u32, 0u32, 1u32, 2u32, 0u32, 1u32, 2u32,
                3u32, 4u32, 5u32, 6u32
            ]
        );
        assert_eq!(batch_buffer.fragmentation_size, 0);

        // reallocate
        batch_buffer.reallocate(&p2, 4);
        {
            let mut mapping = batch_buffer
                .buffer()
                .open(p2.borrow().first, p2.borrow().size);
            for i in 0..4 {
                mapping.set(i, i as usize);
            }
            batch_buffer.buffer().close(mapping);
        }
        assert_eq!(
            &batch_buffer.buffer.indices.borrow()[0..batch_buffer.last()],
            &[
                0u32, 1u32, 2u32, 3u32, 4u32, 5u32, 6u32, 7u32, 0u32, 0u32, 0u32, 0u32, 1u32, 2u32,
                3u32, 4u32, 5u32, 6u32, 0u32, 1u32, 2u32, 3u32
            ]
        );
        assert_eq!(batch_buffer.fragmentation_size, 3);

        batch_buffer.reallocate(&p3, 2);
        {
            let mut mapping = batch_buffer
                .buffer()
                .open(p3.borrow().first, p3.borrow().size);
            for i in 0..2 {
                mapping.set(i, i as usize);
            }
            batch_buffer.buffer().close(mapping);
        }
        assert_eq!(
            &batch_buffer.buffer.indices.borrow()[0..batch_buffer.last()],
            &[
                0u32, 1u32, 2u32, 3u32, 4u32, 5u32, 6u32, 7u32, 0u32, 0u32, 0u32, 0u32, 1u32, 0u32,
                0u32, 0u32, 0u32, 0u32, 0u32, 1u32, 2u32, 3u32
            ]
        );
        assert_eq!(batch_buffer.fragmentation_size, 8);

        batch_buffer.free(&p1);
        assert_eq!(
            &batch_buffer.buffer.indices.borrow()[0..batch_buffer.last()],
            &[
                0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 1u32, 0u32,
                0u32, 0u32, 0u32, 0u32, 0u32, 1u32, 2u32, 3u32
            ]
        );
        assert_eq!(batch_buffer.fragmentation_size, 16);

        let p4 = batch_buffer.allocate(6);
        {
            let mut mapping = batch_buffer
                .buffer()
                .open(p4.borrow().first, p4.borrow().size);
            for i in 0..6 {
                mapping.set(i, i as usize);
            }
            batch_buffer.buffer().close(mapping);
        }
        assert_eq!(
            &batch_buffer.buffer.indices.borrow()[0..batch_buffer.last()],
            &[
                0u32, 1u32, 2u32, 3u32, 4u32, 5u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 1u32, 0u32,
                0u32, 0u32, 0u32, 0u32, 0u32, 1u32, 2u32, 3u32
            ]
        );
        assert_eq!(batch_buffer.fragmentation_size, 10);

        let p5 = batch_buffer.allocate(3);
        {
            let mut mapping = batch_buffer
                .buffer()
                .open(p5.borrow().first, p5.borrow().size);
            for i in 0..3 {
                mapping.set(i, i as usize);
            }
            batch_buffer.buffer().close(mapping);
        }
        assert_eq!(
            &batch_buffer.buffer.indices.borrow()[0..batch_buffer.last()],
            &[
                0u32, 1u32, 2u32, 3u32, 4u32, 5u32, 0u32, 0u32, 0u32, 1u32, 2u32, 0u32, 1u32, 0u32,
                0u32, 0u32, 0u32, 0u32, 0u32, 1u32, 2u32, 3u32
            ]
        );
        assert_eq!(batch_buffer.fragmentation_size, 7);

        batch_buffer.defragmentation();
        assert_eq!(batch_buffer.fragmentation_size, 0);
        assert_eq!(batch_buffer.last(), 15);
    }
}
