use crate::batch::batch_command::BatchObjectId;
use crate::batch::batch_pointer::BatchPointer;
use crate::hal::buffer::buffer_interface::BufferInterface;
use std::collections::HashMap;
use tearchan_utility::btree::DuplicatableBTreeMap;

pub struct BatchBuffer<TBuffer: BufferInterface> {
    buffer: TBuffer,
    buffer_factory: fn(&TBuffer, usize) -> TBuffer,
    pointers: HashMap<BatchObjectId, BatchPointer>,
    last: usize,
    pending_pointers: DuplicatableBTreeMap<usize, BatchPointer>,
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

    pub fn allocate(&mut self, id: BatchObjectId, size: usize) -> &mut BatchPointer {
        debug_assert!(!self.pointers.contains_key(&id));

        // Search from pending_pointers
        if let Some(mut ptr) = match self.pending_pointers.range_mut(size..).next() {
            Some((_, pointers)) => pointers.pop_back(),
            None => None,
        } {
            // Reuse the memory if there is more free space than the desired size
            self.fragmentation_size -= ptr.size; // Note that reduce will increase the fragment size
            if ptr.size != size {
                // Reducing unnecessary memory size
                self.reduce_pointer(&mut ptr, size);
            }

            self.pointers.insert(id, ptr);
        } else {
            // Allocate new memory space
            let ptr = self.allocate_new_pointer(size);
            self.pointers.insert(id, ptr);
        }
        self.pointers.get_mut(&id).unwrap()
    }

    pub fn reallocate(&mut self, id: BatchObjectId, size: usize) {
        let mut pointer = self.pointers.remove(&id).unwrap();
        match pointer.size {
            d if d > size => {
                self.reduce_pointer(&mut pointer, size);
                self.pointers.insert(id, pointer);
            }
            d if d < size => {
                self.buffer.clear(pointer.first, pointer.size);
                self.fragmentation_size += pointer.size;

                self.allocate(id, size);
                self.pending_pointers.push_back(pointer.size, pointer);
            }
            _ => {}
        }
    }

    pub fn free(&mut self, id: BatchObjectId) {
        let pointer = self.pointers.remove(&id).unwrap();
        self.fragmentation_size += pointer.size;
        self.buffer.clear(pointer.first, pointer.size);
        self.pending_pointers.push_back(pointer.size, pointer);
    }

    pub fn buffer(&self) -> &TBuffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut TBuffer {
        &mut self.buffer
    }

    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    pub fn last(&self) -> usize {
        self.last
    }

    pub fn fragmentation_size(&self) -> usize {
        self.fragmentation_size
    }

    // NOTICE: Destroy structures
    pub fn defragmentation(&mut self) {
        let mut first: usize = 0;
        for (_, pointer) in &mut self.pointers {
            pointer.first = first;
            first += pointer.size;
        }

        self.last = first;
        self.pending_pointers.clear();
        self.fragmentation_size = 0;
    }

    pub fn get_pointer(&self, id: &BatchObjectId) -> Option<&BatchPointer> {
        self.pointers.get(id)
    }

    fn reallocate_buffer(&mut self, size: usize) {
        let new_size = size * 2;
        let factory = &self.buffer_factory;
        self.buffer = factory(&self.buffer, new_size);
    }

    fn allocate_new_pointer(&mut self, size: usize) -> BatchPointer {
        let first = self.last;
        if first + size > self.buffer.len() {
            self.reallocate_buffer(first + size);
        }

        self.last += size;
        BatchPointer::new(first, size)
    }

    fn reduce_pointer(&mut self, pointer: &mut BatchPointer, size: usize) {
        if pointer.last() != self.last {
            let r_first = pointer.first + size;
            let r_size = pointer.size - size;
            let r_ptr = BatchPointer::new(r_first, r_size);
            self.pending_pointers.push_back(r_size, r_ptr);

            self.buffer.clear(r_first, r_size);
            self.fragmentation_size += r_size;
        } else {
            self.last = pointer.first + size;
        }

        pointer.size = size;
    }
}

#[cfg(test)]
mod test {
    use crate::batch::batch_buffer::BatchBuffer;
    use crate::hal::buffer::buffer_interface::{BufferInterface, BufferMappedMemoryInterface};
    use crate::hal::buffer::index_buffer::test::MockIndexBuffer;
    use crate::hal::buffer::vertex_buffer::test::MockVertexBuffer;
    use tearchan_utility::shared::Shared;
    use tearchan_utility::test::mock::MockFunc;

    #[test]
    fn test_batch_buffer_memory_allocation() {
        let mock = Shared::new(MockFunc::default());
        let mock_vertex_buffer = MockVertexBuffer::new(&mock, vec![0.0f32; 32].as_slice());
        let mut batch_buffer = BatchBuffer::new(mock_vertex_buffer, |buffer, size| {
            let mut vertices = buffer.vertices.borrow().clone();
            vertices.resize(size, 0.0f32);
            MockVertexBuffer::new(&buffer.mock, &vertices)
        });

        {
            let p1 = batch_buffer.allocate(1, 30);
            assert_eq!(p1.first, 0);
            assert_eq!(p1.last(), 30);
        }
        assert_eq!(batch_buffer.pointers.len(), 1);

        {
            let p2 = batch_buffer.allocate(2, 50);
            assert_eq!(p2.first, 30);
            assert_eq!(p2.last(), 80);
        }
        assert_eq!(batch_buffer.pointers.len(), 2);

        {
            let p3 = batch_buffer.allocate(3, 42);
            assert_eq!(p3.first, 80);
            assert_eq!(p3.last(), 122);
        }
        assert_eq!(batch_buffer.pointers.len(), 3);

        assert_eq!(batch_buffer.size(), 160); // (30 + 50) * 2
        assert_eq!(batch_buffer.last(), 122);

        batch_buffer.reallocate(1, 40);
        assert_eq!(batch_buffer.get_pointer(&1).unwrap().first, 122);
        assert_eq!(batch_buffer.get_pointer(&1).unwrap().last(), 162);
        assert_eq!(batch_buffer.get_pointer(&2).unwrap().first, 30);
        assert_eq!(batch_buffer.get_pointer(&2).unwrap().last(), 80);
        assert_eq!(batch_buffer.get_pointer(&3).unwrap().first, 80);
        assert_eq!(batch_buffer.get_pointer(&3).unwrap().last(), 122);

        assert_eq!(batch_buffer.size(), 324); // (30 + 50) * 2 => 162 * 2
        assert_eq!(batch_buffer.last(), 162);
        assert_eq!(batch_buffer.pending_pointers.len(), 1);

        batch_buffer.free(2);
        assert_eq!(batch_buffer.get_pointer(&1).unwrap().first, 122);
        assert_eq!(batch_buffer.get_pointer(&1).unwrap().last(), 162);
        assert_eq!(batch_buffer.get_pointer(&3).unwrap().first, 80);
        assert_eq!(batch_buffer.get_pointer(&3).unwrap().last(), 122);
        assert_eq!(batch_buffer.size(), 324);
        assert_eq!(batch_buffer.last(), 162);
        assert_eq!(batch_buffer.pointers.len(), 2);
        assert_eq!(batch_buffer.pending_pointers.len(), 2);

        batch_buffer.reallocate(3, 40);
        assert_eq!(batch_buffer.get_pointer(&3).unwrap().first, 80);
        assert_eq!(batch_buffer.get_pointer(&3).unwrap().last(), 120);
        assert_eq!(batch_buffer.last(), 162);
        assert_eq!(batch_buffer.pointers.len(), 2);
        assert_eq!(batch_buffer.pending_pointers.len(), 3);

        batch_buffer.reallocate(1, 30);
        assert_eq!(batch_buffer.get_pointer(&1).unwrap().first, 122);
        assert_eq!(batch_buffer.get_pointer(&1).unwrap().last(), 152);
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
        let mock = Shared::new(MockFunc::new());
        let mock_vertex_buffer = MockIndexBuffer::new(&mock, vec![0u32; 32].as_slice());
        let mut batch_buffer = BatchBuffer::new(mock_vertex_buffer, |buffer, size| {
            let mut indices = buffer.indices.borrow().clone();
            indices.resize(size, 0u32);
            MockIndexBuffer::new(&buffer.mock, &indices)
        });

        let p1 = batch_buffer.allocate(1, 8).clone();
        {
            let mut mapping = batch_buffer.buffer().open(p1.first, p1.size);
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

        let p2 = batch_buffer.allocate(2, 3).clone();
        {
            let mut mapping = batch_buffer.buffer().open(p2.first, p2.size);
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

        let p3 = batch_buffer.allocate(3, 7).clone();
        {
            let mut mapping = batch_buffer.buffer().open(p3.first, p3.size);
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
        batch_buffer.reallocate(2, 4);
        let p2 = batch_buffer.get_pointer(&2).unwrap();
        {
            let mut mapping = batch_buffer.buffer().open(p2.first, p2.size);
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

        batch_buffer.reallocate(3, 2);
        let p3 = batch_buffer.get_pointer(&3).unwrap();
        {
            let mut mapping = batch_buffer.buffer().open(p3.first, p3.size);
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

        batch_buffer.free(1);
        assert_eq!(
            &batch_buffer.buffer.indices.borrow()[0..batch_buffer.last()],
            &[
                0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 1u32, 0u32,
                0u32, 0u32, 0u32, 0u32, 0u32, 1u32, 2u32, 3u32
            ]
        );
        assert_eq!(batch_buffer.fragmentation_size, 16);

        let p4 = batch_buffer.allocate(4, 6).clone();
        {
            let mut mapping = batch_buffer.buffer().open(p4.first, p4.size);
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

        let p5 = batch_buffer.allocate(5, 3).clone();
        {
            let mut mapping = batch_buffer.buffer().open(p5.first, p5.size);
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
