use crate::buffer::BufferTrait;
use std::cmp::Ordering;
use std::collections::{BTreeMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use tearchan_util::btree::DuplicatableBTreeMap;

#[derive(Debug)]
pub enum BatchBufferAllocatorEvent {
    Write(BatchBufferPointer),
    Clear(BatchBufferPointer),
    ReallocateAll {
        pairs: Vec<BatchBufferAllocatorReallocPair>,
    },
}

#[derive(Debug)]
pub struct BatchBufferAllocatorReallocPair {
    pub from: BatchBufferPointer,
    pub to: BatchBufferPointer,
}

#[derive(Debug)]
enum Event {
    Clear {
        pointer: BatchBufferPointer,
    },
    Write {
        first: usize,
    },
    Reallocate {
        pairs: Vec<BatchBufferAllocatorReallocPair>,
    },
}

#[derive(Default)]
pub struct BatchBufferAllocator {
    pointers: BTreeMap<usize, BatchBufferPointer>, // grouped by first
    pending_pointers: DuplicatableBTreeMap<usize, usize>, // grouped by size, value is last of pending pointer
    pending_pointers_grouped_by_last: BTreeMap<usize, BatchBufferPointer>,
    events: VecDeque<Event>,
    len: usize,
}

impl BatchBufferAllocator {
    pub fn allocate(&mut self, len: usize) -> BatchBufferPointer {
        if let Some(pointer) = self.allocate_from_pending_pointers(len) {
            self.events.push_back(Event::Write {
                first: pointer.first,
            });
            return pointer;
        }
        let first = self
            .pointers
            .last_key_value()
            .map(|(key, pointer)| *key + pointer.len)
            .unwrap_or(0);
        let pointer = BatchBufferPointer::new(first, len);
        self.pointers.insert(first, pointer);
        self.len += len;
        self.events.push_back(Event::Write { first });
        pointer
    }

    pub fn reallocate(&mut self, pointer: BatchBufferPointer, size: usize) -> BatchBufferPointer {
        self.free(pointer);
        self.allocate(size)
    }

    pub fn free(&mut self, pointer: BatchBufferPointer) {
        let first = pointer.first;
        let last = pointer.first + pointer.len;
        self.pointers.remove(&pointer.first);

        let is_last = first == self.len - pointer.len;
        let mut seek = first;

        let mut merge_pointers = vec![pointer];
        for (last, pointer) in self.pending_pointers_grouped_by_last.range(0..=first).rev() {
            if &seek != last {
                break;
            }
            merge_pointers.push(*pointer);
            seek -= pointer.len;
        }

        for merge_pointer in merge_pointers {
            let last = merge_pointer.first + merge_pointer.len;
            self.pending_pointers.remove(&merge_pointer.len, &last);
            self.pending_pointers_grouped_by_last.remove(&last);
        }

        if is_last {
            self.len = seek;
            return;
        }

        self.push_pending_pointer(BatchBufferPointer::new(seek, last - seek));
        self.events.push_back(Event::Clear { pointer });
    }

    pub fn defragmentation(&mut self) {
        self.pending_pointers.clear();
        self.pending_pointers_grouped_by_last.clear();
        self.events.clear();

        let prev_pointers = std::mem::take(&mut self.pointers);
        let mut seek = 0;
        let mut pairs = Vec::new();
        let mut write_events = VecDeque::new();
        for (_, pointer) in prev_pointers.into_iter() {
            let len = pointer.len;
            let new_pointer = BatchBufferPointer::new(seek, len);
            self.pointers.insert(seek, new_pointer);

            write_events.push_back(Event::Write {
                first: new_pointer.first,
            });
            pairs.push(BatchBufferAllocatorReallocPair {
                from: pointer,
                to: new_pointer,
            });

            seek += len;
        }

        self.events.push_back(Event::Reallocate { pairs });
        self.events.append(&mut write_events);

        self.len = seek;
    }

    pub fn pop_event(&mut self) -> Option<BatchBufferAllocatorEvent> {
        match self.events.pop_front()? {
            Event::Write { first } => {
                let pointer = self.pointers.get(&first)?;
                Some(BatchBufferAllocatorEvent::Write(*pointer))
            }
            Event::Clear { pointer } => Some(BatchBufferAllocatorEvent::Clear(pointer)),
            Event::Reallocate { pairs } => Some(BatchBufferAllocatorEvent::ReallocateAll { pairs }),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&BatchBufferPointer, &BatchBufferPointer) -> Ordering,
    {
        let mut pointers = self.pointers.iter().map(|(_, p)| *p).collect::<Vec<_>>();
        pointers.sort_by(compare);

        self.events.clear();
        self.pointers.clear();
        self.pending_pointers.clear();
        self.pending_pointers_grouped_by_last.clear();

        let mut pairs = Vec::new();
        let mut write_events = VecDeque::new();
        let mut seek = 0;
        for pointer in pointers {
            let new_pointer = BatchBufferPointer::new(seek, pointer.len);
            self.pointers.insert(new_pointer.first, new_pointer);

            write_events.push_back(Event::Write {
                first: new_pointer.first,
            });
            pairs.push(BatchBufferAllocatorReallocPair {
                from: pointer,
                to: new_pointer,
            });

            seek += new_pointer.len;
        }

        self.events.push_back(Event::Reallocate { pairs });
        self.events.append(&mut write_events);

        assert!(seek <= self.len, "{} <= {}", seek, self.len);
        self.len = seek;
    }

    fn allocate_from_pending_pointers(&mut self, len: usize) -> Option<BatchBufferPointer> {
        let last = self
            .pending_pointers
            .range_mut(len..)
            .next()?
            .1
            .pop_front()?;
        let pointer = self.pending_pointers_grouped_by_last.remove(&last).unwrap();
        {
            let (pointer, pending_pointer) = if len == pointer.len {
                (pointer, None)
            } else {
                (
                    BatchBufferPointer::new(pointer.first, len),
                    Some(BatchBufferPointer::new(
                        pointer.first + len,
                        pointer.len - len,
                    )),
                )
            };
            self.pointers.insert(pointer.first, pointer);
            if let Some(pending_pointer) = pending_pointer {
                self.push_pending_pointer(pending_pointer);
            }
            Some(pointer)
        }
    }

    fn push_pending_pointer(&mut self, pointer: BatchBufferPointer) {
        let last = pointer.first + pointer.len;
        self.pending_pointers_grouped_by_last.insert(last, pointer);
        self.pending_pointers.push_front(pointer.len, last);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BatchBufferPointer {
    first: usize,
    len: usize,
}

impl Eq for BatchBufferPointer {}

impl PartialEq for BatchBufferPointer {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first
    }
}

impl Ord for BatchBufferPointer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.first.cmp(&other.first)
    }
}

impl PartialOrd for BatchBufferPointer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.first.partial_cmp(&other.first)
    }
}

impl Hash for BatchBufferPointer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.first)
    }
}

impl BatchBufferPointer {
    fn new(first: usize, len: usize) -> Self {
        BatchBufferPointer { first, len }
    }

    pub fn first(&self) -> usize {
        self.first
    }
}

pub struct BatchBuffer<TBuffer, TDataType> {
    buffer: TBuffer,
    _phantom: PhantomData<TDataType>,
}

impl<'a, TBuffer, TDataType> BatchBuffer<TBuffer, TDataType>
where
    TBuffer: BufferTrait<'a, TDataType>,
{
    pub fn new(buffer: TBuffer) -> Self {
        Self {
            buffer,
            _phantom: PhantomData,
        }
    }

    pub fn write(
        &mut self,
        writer: TBuffer::Writer,
        pointer: BatchBufferPointer,
        data: &[TDataType],
    ) {
        assert!(pointer.len <= data.len());
        self.buffer.write(writer, data, pointer.first);
    }

    pub fn clear(&mut self, writer: TBuffer::Writer, pointer: BatchBufferPointer) {
        self.buffer.clear(writer, pointer.first, pointer.len);
    }

    pub fn resize(&mut self, resizer: TBuffer::Resizer, len: usize) {
        self.buffer.resize(resizer, len);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn buffer(&self) -> &TBuffer {
        &self.buffer
    }
}

#[cfg(test)]
mod test {
    use crate::batch::buffer::{
        BatchBuffer, BatchBufferAllocator, BatchBufferAllocatorEvent, BatchBufferPointer,
    };
    use crate::buffer::BufferTrait;
    use std::collections::HashMap;

    struct ResultContext;

    #[derive(Default)]
    struct VecBuffer {
        data: Vec<u32>,
    }

    impl VecBuffer {
        pub fn new(len: usize) -> VecBuffer {
            VecBuffer { data: vec![0; len] }
        }
    }

    impl<'a> BufferTrait<'a, u32> for VecBuffer {
        type Resizer = &'a mut ResultContext;
        type Writer = &'a mut ResultContext;
        type Copier = &'a mut ResultContext;

        fn resize(&mut self, _resizer: Self::Resizer, len: usize) {
            self.data.resize(len, 0);
        }

        fn write(&mut self, _writer: &mut ResultContext, data: &[u32], offset: usize) {
            self.data
                .splice(offset..(offset + data.len()), data.iter().copied());
        }

        fn copy(&mut self, _copy: &mut ResultContext, from: usize, to: usize, len: usize) {
            let from = {
                self.data.as_slice()[from..(from + len)]
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
            };
            self.data.splice(to..(to + len), from);
        }

        fn len(&self) -> usize {
            self.data.len()
        }

        fn is_empty(&self) -> bool {
            self.data.is_empty()
        }

        fn clear(&mut self, _writer: &mut ResultContext, offset: usize, len: usize) {
            self.data.splice(offset..(offset + len), vec![0; len]);
        }
    }

    fn v<T>(size: usize, value: T) -> Vec<T>
    where
        T: Clone,
    {
        let mut v = Vec::with_capacity(size);
        v.resize(size, value);
        v
    }

    fn convert_events(allocator: &mut BatchBufferAllocator) -> Vec<BatchBufferAllocatorEvent> {
        let mut events = Vec::new();
        while let Some(event) = allocator.pop_event() {
            events.push(event);
        }
        events
    }

    #[test]
    fn test_basic_write() {
        let mut allocator = BatchBufferAllocator::default();
        let mut index_buffer = BatchBuffer::new(VecBuffer::new(10));
        let mut sprites = HashMap::new();
        let mut context = ResultContext;

        let p0 = allocator.allocate(5);
        sprites.insert(p0, 1);

        let allocator_len = allocator.len();
        while let Some(event) = allocator.pop_event() {
            match event {
                BatchBufferAllocatorEvent::Clear(pointer) => {
                    index_buffer.clear(&mut context, pointer);
                }
                BatchBufferAllocatorEvent::Write(pointer) => {
                    if index_buffer.len() < allocator_len {
                        index_buffer.resize(&mut context, allocator_len * 2);
                    }
                    let sprite = sprites.get(&pointer).unwrap();
                    index_buffer.write(&mut context, pointer, &v(pointer.len, *sprite));
                }
                BatchBufferAllocatorEvent::ReallocateAll { pairs } => {
                    for pair in pairs.iter() {
                        sprites.remove(&pair.from);
                    }
                    for (sprite, pointer) in pairs
                        .iter()
                        .filter_map(|pair| {
                            sprites.remove(&pair.from).map(|sprite| (sprite, pair.to))
                        })
                        .collect::<Vec<_>>()
                    {
                        sprites.insert(pointer, sprite);
                    }
                }
            }
        }

        assert_eq!(
            index_buffer.buffer.data[0..allocator.len()],
            [1, 1, 1, 1, 1]
        );

        let p1 = allocator.allocate(3);
        sprites.insert(p1, 2);

        let p2 = allocator.allocate(4);
        sprites.insert(p2, 3);

        let allocator_len = allocator.len();
        while let Some(event) = allocator.pop_event() {
            match event {
                BatchBufferAllocatorEvent::Clear(pointer) => {
                    index_buffer.clear(&mut context, pointer);
                }
                BatchBufferAllocatorEvent::Write(pointer) => {
                    if index_buffer.len() < allocator_len {
                        index_buffer.resize(&mut context, allocator_len * 2);
                    }
                    let sprite = sprites.get(&pointer).unwrap();
                    index_buffer.write(&mut context, pointer, &v(pointer.len, *sprite));
                }
                BatchBufferAllocatorEvent::ReallocateAll { pairs } => {
                    for pair in pairs.iter() {
                        sprites.remove(&pair.from);
                    }
                    for (sprite, pointer) in pairs
                        .iter()
                        .filter_map(|pair| {
                            sprites.remove(&pair.from).map(|sprite| (sprite, pair.to))
                        })
                        .collect::<Vec<_>>()
                    {
                        sprites.insert(pointer, sprite);
                    }
                }
            }
        }

        assert_eq!(
            index_buffer.buffer.data[0..allocator.len()],
            [1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3]
        );
    }

    #[test]
    fn test_free() {
        let mut allocator = BatchBufferAllocator::default();

        let p0 = allocator.allocate(1); // 5
        let p1 = allocator.allocate(2); // 1
        let p2 = allocator.allocate(4); // 2
        let p3 = allocator.allocate(3); // 4
        let p4 = allocator.allocate(5); // 3

        assert_eq!(allocator.len(), 15);

        allocator.free(p1);

        assert_eq!(allocator.len(), 15);
        assert_eq!(
            allocator.pending_pointers.len(),
            1,
            "{:?}",
            allocator.pending_pointers
        );
        assert_eq!(
            allocator.pending_pointers_grouped_by_last.len(),
            1,
            "{:?}",
            allocator.pending_pointers_grouped_by_last
        );

        allocator.free(p2);

        assert_eq!(allocator.len(), 15);
        assert_eq!(
            allocator.pending_pointers.len(),
            1,
            "{:?}",
            allocator.pending_pointers
        );
        assert_eq!(
            allocator.pending_pointers_grouped_by_last.len(),
            1,
            "{:?}",
            allocator.pending_pointers_grouped_by_last
        );

        allocator.free(p4);

        assert_eq!(allocator.len(), 10);
        assert_eq!(
            allocator.pending_pointers.len(),
            1,
            "{:?}",
            allocator.pending_pointers
        );
        assert_eq!(
            allocator.pending_pointers_grouped_by_last.len(),
            1,
            "{:?}",
            allocator.pending_pointers_grouped_by_last
        );

        allocator.free(p3);

        assert_eq!(allocator.len(), 1);
        assert_eq!(
            allocator.pending_pointers.len(),
            0,
            "{:?}",
            allocator.pending_pointers
        );
        assert_eq!(
            allocator.pending_pointers_grouped_by_last.len(),
            0,
            "{:?}",
            allocator.pending_pointers_grouped_by_last
        );

        allocator.free(p0);

        assert_eq!(allocator.len(), 0);
        assert_eq!(
            allocator.pending_pointers.len(),
            0,
            "{:?}",
            allocator.pending_pointers
        );
        assert_eq!(
            allocator.pending_pointers_grouped_by_last.len(),
            0,
            "{:?}",
            allocator.pending_pointers_grouped_by_last
        );
    }

    #[test]
    fn test_allocate() {
        let mut allocator = BatchBufferAllocator::default();

        let p0 = allocator.allocate(3);
        assert_eq!(p0.first, 0);
        assert_eq!(p0.len, 3);

        let p1 = allocator.allocate(4);
        assert_eq!(p1.first, 3);
        assert_eq!(p1.len, 4);

        let p2 = allocator.allocate(5);
        assert_eq!(p2.first, 7);
        assert_eq!(p2.len, 5);

        assert_eq!(allocator.pointers.len(), 3);
        assert_eq!(allocator.len(), 12);

        insta::assert_debug_snapshot!(convert_events(&mut allocator));
    }

    #[test]
    fn test_reallocate() {
        let mut allocator = BatchBufferAllocator::default();
        let p0 = allocator.allocate(10);
        let p1 = allocator.allocate(5);
        let _p2 = allocator.allocate(15);
        let p1 = allocator.reallocate(p1, 10);
        assert_eq!(p1.first, 30);
        assert_eq!(p1.len, 10);

        let p0 = allocator.reallocate(p0, 5);
        assert_eq!(p0.first, 10);
        assert_eq!(p0.len, 5);
    }

    #[test]
    fn test_defragmentation() {
        let mut allocator = BatchBufferAllocator::default();
        let p0 = allocator.allocate(3);
        let p1 = allocator.allocate(3);
        let _p2 = allocator.allocate(3);
        let p3 = allocator.allocate(3);
        let _p4 = allocator.allocate(3);

        allocator.free(p0);
        allocator.free(p1);
        allocator.free(p3);

        convert_events(&mut allocator);

        allocator.defragmentation();
        assert_eq!(allocator.pointers.len(), 2);
        assert_eq!(allocator.pending_pointers.len(), 0);
        assert_eq!(allocator.pending_pointers_grouped_by_last.len(), 0);
        assert_eq!(allocator.len, 6);

        insta::assert_debug_snapshot!(convert_events(&mut allocator));

        assert_eq!(allocator.pointers.get(&0).unwrap().len, 3);
        assert_eq!(allocator.pointers.get(&3).unwrap().len, 3);
    }

    #[test]
    fn test_events() {
        let mut allocator = BatchBufferAllocator::default();

        let p0 = allocator.allocate(1); // 4
        let p1 = allocator.allocate(2); // 1
        let p2 = allocator.allocate(4); // 3
        let _p3 = allocator.allocate(3);
        let p4 = allocator.allocate(5); // 2

        insta::assert_debug_snapshot!(convert_events(&mut allocator));

        allocator.free(p1);

        insta::assert_debug_snapshot!(convert_events(&mut allocator));

        allocator.free(p4);

        insta::assert_debug_snapshot!(convert_events(&mut allocator));

        allocator.free(p2);

        insta::assert_debug_snapshot!(convert_events(&mut allocator));

        let _p0 = allocator.reallocate(p0, 5);

        insta::assert_debug_snapshot!(convert_events(&mut allocator));
    }

    #[test]
    fn test_sort() {
        let mut allocator = BatchBufferAllocator::default();

        // 5, 3, 4, 1
        let _p0 = allocator.allocate(5);
        let _p1 = allocator.allocate(3);
        let _p2 = allocator.allocate(4);
        let _p3 = allocator.allocate(1);

        allocator.sort_by(|p0, p1| p0.len.cmp(&p1.len)); // 1, 3, 4, 5

        insta::assert_debug_snapshot!(convert_events(&mut allocator));

        let p1 = BatchBufferPointer::new(1, 3);
        allocator.free(p1); // 1, _, 4, 5

        allocator.allocate(2); // 1, 2, 4, 5

        allocator.sort_by(|p0, p1| p0.len.cmp(&p1.len));

        insta::assert_debug_snapshot!(convert_events(&mut allocator));

        assert_eq!(allocator.len, 12);
    }
}
