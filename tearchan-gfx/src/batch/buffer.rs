use crate::batch::BatchObjectId;
use crate::buffer::BufferInterface;
use std::collections::HashMap;
use tearchan_util::btree::DuplicatableBTreeMap;

const DEFAULT_BUFFER_SIZE: usize = 1024usize;
pub type BufferFactory<TBuffer> = fn(
    &<TBuffer as BufferInterface>::Device,
    &<TBuffer as BufferInterface>::Queue,
    &mut Option<&mut <TBuffer as BufferInterface>::Encoder>,
    Option<(&TBuffer, usize)>, // (previous buffer, previous buffer used size)
    usize,
) -> TBuffer;

#[derive(Debug, Clone)]
pub struct BatchPointer {
    pub first: usize,
    pub size: usize,
}

impl BatchPointer {
    pub fn new(first: usize, size: usize) -> Self {
        BatchPointer { first, size }
    }

    pub fn last(&self) -> usize {
        self.first + self.size
    }
}

pub struct BatchBuffer<TBuffer>
where
    TBuffer: BufferInterface,
{
    buffer: TBuffer,
    buffer_factory: BufferFactory<TBuffer>,
    pointers: HashMap<BatchObjectId, BatchPointer>,
    last: usize,
    flushed_last: usize,
    pending_pointers: DuplicatableBTreeMap<usize, BatchPointer>,
    fragmentation_size: usize,
}

impl<TBuffer> BatchBuffer<TBuffer>
where
    TBuffer: BufferInterface,
{
    pub fn new(
        device: &TBuffer::Device,
        queue: &TBuffer::Queue,
        encoder: &mut Option<&mut TBuffer::Encoder>,
        buffer_factory: BufferFactory<TBuffer>,
    ) -> Self {
        let factory = &buffer_factory;
        BatchBuffer {
            buffer: factory(device, queue, encoder, None, DEFAULT_BUFFER_SIZE),
            buffer_factory,
            pointers: HashMap::new(),
            last: 0,
            flushed_last: 0,
            pending_pointers: DuplicatableBTreeMap::default(),
            fragmentation_size: 0,
        }
    }

    pub fn allocate(
        &mut self,
        device: &TBuffer::Device,
        queue: &TBuffer::Queue,
        encoder: &mut Option<&mut TBuffer::Encoder>,
        id: BatchObjectId,
        size: usize,
    ) -> &mut BatchPointer {
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
                self.reduce_pointer(queue, &mut ptr, size);
            }

            self.pointers.insert(id, ptr);
        } else {
            // Allocate new memory space
            let ptr = self.allocate_new_pointer(device, queue, encoder, size);
            self.pointers.insert(id, ptr);
        }
        self.pointers.get_mut(&id).unwrap()
    }

    pub fn reallocate(
        &mut self,
        device: &TBuffer::Device,
        queue: &TBuffer::Queue,
        encoder: &mut Option<&mut TBuffer::Encoder>,
        id: BatchObjectId,
        size: usize,
    ) {
        let mut pointer = self.pointers.remove(&id).unwrap();
        match pointer.size {
            d if d > size => {
                self.reduce_pointer(queue, &mut pointer, size);
                self.pointers.insert(id, pointer);
            }
            d if d < size => {
                self.buffer.clear(queue, pointer.first, pointer.size);
                self.fragmentation_size += pointer.size;

                self.allocate(device, queue, encoder, id, size);
                self.pending_pointers.push_back(pointer.size, pointer);
            }
            _ => {
                self.pointers.insert(id, pointer);
            }
        }
    }

    pub fn free(&mut self, queue: &TBuffer::Queue, id: BatchObjectId) {
        let pointer = self.pointers.remove(&id).unwrap();
        self.fragmentation_size += pointer.size;
        self.buffer.clear(queue, pointer.first, pointer.size);
        self.pending_pointers.push_back(pointer.size, pointer);
    }

    pub fn write(
        &self,
        queue: &TBuffer::Queue,
        pointer: &BatchPointer,
        data: &[TBuffer::DataType],
    ) {
        assert!(data.len() <= pointer.size);
        self.buffer
            .write(queue, bytemuck::cast_slice(data), pointer.first);
    }

    pub fn buffer(&self) -> &TBuffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
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
        for pointer in &mut self.pointers {
            pointer.1.first = first;
            first += pointer.1.size;
        }

        self.last = first;
        self.pending_pointers.clear();
        self.fragmentation_size = 0;
    }

    pub fn get_pointer(&self, id: &BatchObjectId) -> Option<&BatchPointer> {
        self.pointers.get(id)
    }

    pub fn sort(&mut self, ids: &[BatchObjectId]) {
        debug_assert_eq!(ids.len(), self.pointers.len());
        let mut pointers = HashMap::new();
        let mut size = 0;
        for id in ids {
            let pointer = &self.pointers[id];
            pointers.insert(*id, BatchPointer::new(size, pointer.size));
            size += pointer.size;
        }

        self.pointers = pointers;
        self.last = size;
    }

    fn reallocate_buffer(
        &mut self,
        device: &TBuffer::Device,
        queue: &TBuffer::Queue,
        encoder: &mut Option<&mut TBuffer::Encoder>,
        size: usize,
    ) {
        let new_size = size * 2;
        let factory = &self.buffer_factory;
        self.buffer = factory(
            device,
            queue,
            encoder,
            Some((&self.buffer, self.flushed_last)),
            new_size,
        );
    }

    fn allocate_new_pointer(
        &mut self,
        device: &TBuffer::Device,
        queue: &TBuffer::Queue,
        encoder: &mut Option<&mut TBuffer::Encoder>,
        size: usize,
    ) -> BatchPointer {
        let first = self.last;
        if first + size > self.buffer.len() {
            self.reallocate_buffer(device, queue, encoder, first + size);
        }

        self.last += size;
        BatchPointer::new(first, size)
    }

    fn reduce_pointer(&mut self, queue: &TBuffer::Queue, pointer: &mut BatchPointer, size: usize) {
        if pointer.last() != self.last {
            let r_first = pointer.first + size;
            let r_size = pointer.size - size;
            let r_ptr = BatchPointer::new(r_first, r_size);
            self.pending_pointers.push_back(r_size, r_ptr);

            self.buffer.clear(queue, r_first, r_size);
            self.fragmentation_size += r_size;
        } else {
            self.last = pointer.first + size;
        }

        pointer.size = size;
    }

    pub fn flush(&mut self) {
        // Update the size copied to the buffer.
        // This will determine how much of the existing buffer should be restored when the buffer is recreated.
        self.flushed_last = self.last;
    }
}

#[cfg(test)]
mod test {
    use crate::batch::buffer::BatchBuffer;
    use crate::buffer::BufferInterface;
    use std::cell::RefCell;

    struct MockBuffer(RefCell<Vec<u32>>);

    impl BufferInterface for MockBuffer {
        type DataType = u32;
        type Device = ();
        type Queue = ();
        type Encoder = ();

        fn write(&self, _queue: &Self::Queue, data: &[Self::DataType], offset: usize) {
            self.0.borrow_mut()[offset..(offset + data.len())].clone_from_slice(data);
        }

        fn len(&self) -> usize {
            self.0.borrow().len()
        }

        fn is_empty(&self) -> bool {
            self.0.borrow().is_empty()
        }

        fn clear(&self, _queue: &Self::Queue, offset: usize, len: usize) {
            self.0.borrow_mut()[offset..(offset + len)].fill(0);
        }
    }

    #[test]
    fn test_allocate() {
        let mut buffer: BatchBuffer<MockBuffer> =
            BatchBuffer::new(&(), &(), &mut None, |_, _, _, prev, len| {
                MockBuffer(RefCell::new(if let Some((prev_buffer, prev_len)) = prev {
                    let mut v = prev_buffer.0.borrow().clone();
                    for _ in prev_len..len {
                        v.push(0);
                    }
                    v
                } else {
                    vec![0; len]
                }))
            });
        let p0 = buffer.allocate(&(), &(), &mut None, 1, 10).clone();
        buffer.write(&(), &p0, &[1; 10]);
        assert_eq!(p0.first, 0);
        assert_eq!(p0.size, 10);

        let p1 = buffer.allocate(&(), &(), &mut None, 2, 15).clone();
        buffer.write(&(), &p1, &[2; 15]);
        assert_eq!(p1.first, 10);
        assert_eq!(p1.size, 15);

        let p2 = buffer.allocate(&(), &(), &mut None, 3, 5).clone();
        buffer.write(&(), &p2, &[3; 5]);
        assert_eq!(p2.first, 25);
        assert_eq!(p2.size, 5);

        let p3 = buffer.allocate(&(), &(), &mut None, 4, 12).clone();
        buffer.write(&(), &p3, &[4; 12]);
        assert_eq!(p3.first, 30);
        assert_eq!(p3.size, 12);

        buffer.flush();

        assert_eq!(
            &buffer.buffer.0.borrow()[0..buffer.last()],
            &vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3,
                3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4
            ]
        );
    }

    #[test]
    fn test_free() {
        let mut buffer: BatchBuffer<MockBuffer> =
            BatchBuffer::new(&(), &(), &mut None, |_, _, _, prev, len| {
                MockBuffer(RefCell::new(if let Some((prev_buffer, prev_len)) = prev {
                    let mut v = prev_buffer.0.borrow().clone();
                    for _ in prev_len..len {
                        v.push(0);
                    }
                    v
                } else {
                    vec![0; len]
                }))
            });
        let p0 = buffer.allocate(&(), &(), &mut None, 1, 10).clone();
        buffer.write(&(), &p0, &[1; 10]);

        let p1 = buffer.allocate(&(), &(), &mut None, 2, 15).clone();
        buffer.write(&(), &p1, &[2; 15]);

        let p2 = buffer.allocate(&(), &(), &mut None, 3, 5).clone();
        buffer.write(&(), &p2, &[3; 5]);

        let p3 = buffer.allocate(&(), &(), &mut None, 4, 12).clone();
        buffer.write(&(), &p3, &[4; 12]);
        buffer.flush();

        buffer.free(&(), 1);
        buffer.free(&(), 2);
        buffer.free(&(), 3);
        buffer.free(&(), 4);
        assert_eq!(&buffer.buffer.0.borrow()[0..buffer.last()], &vec![0; 42]);

        let p4 = buffer.allocate(&(), &(), &mut None, 5, 16).clone();
        buffer.write(&(), &p4, &[5; 16]);
        assert_eq!(p4.first, 42);
        assert_eq!(p4.size, 16);

        let p5 = buffer.allocate(&(), &(), &mut None, 6, 12).clone();
        buffer.write(&(), &p5, &[6; 12]);
        assert_eq!(p5.first, 30);
        assert_eq!(p5.size, 12);

        let p6 = buffer.allocate(&(), &(), &mut None, 7, 5).clone();
        buffer.write(&(), &p6, &[7; 5]);
        assert_eq!(p6.first, 25);
        assert_eq!(p6.size, 5);

        assert_eq!(
            &buffer.buffer.0.borrow()[0..buffer.last()],
            &vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7,
                7, 7, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
                5, 5
            ]
        );
    }
}
