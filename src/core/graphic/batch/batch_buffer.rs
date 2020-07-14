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
            self.fragmentation_size -= size;

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
                let new_ptr = {
                    let (_, ptr) = self.allocate_new_pointer(size);
                    ptr
                };
                {
                    let mut new_ptr_borrow_mut = new_ptr.borrow_mut();

                    // Swap
                    old_ptr.first = new_ptr_borrow_mut.first;
                    old_ptr.size = new_ptr_borrow_mut.size;
                    new_ptr_borrow_mut.first = old_first;
                    new_ptr_borrow_mut.size = old_size;

                    self.fragmentation_size += old_size;
                }
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

    fn reallocate_buffer(&mut self, size: usize) {
        let new_size = size * 2;
        let factory = &self.buffer_factory;
        self.buffer = factory(&self.buffer, new_size);
    }

    fn defragmentation(&mut self) {
        let mut cloned_pointers: Vec<_> = self
            .pointers
            .values()
            .map(|pointer| clone_shared(pointer))
            .collect();
        cloned_pointers.sort_unstable_by(|a, b| a.borrow().first.cmp(&b.borrow().first));

        let mut first: usize = 0;
        for pointer in cloned_pointers {
            self.buffer.copy_to(pointer.borrow().first, first, pointer.borrow().size);
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
}
