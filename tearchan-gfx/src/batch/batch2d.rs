use crate::batch::buffer::BatchBuffer;
use crate::batch::{Batch, BatchObjectManager, BatchProvider, BatchProviderCommand};
use crate::buffer::{Buffer, BufferInterface};
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use tearchan_util::bytes::flatten;

pub const BATCH2D_ATTRIBUTE_INDEX: usize = 0;
pub const BATCH2D_ATTRIBUTE_POSITION: usize = 1;
pub const BATCH2D_ATTRIBUTE_TEXCOORD: usize = 2;
pub const BATCH2D_ATTRIBUTE_COLOR: usize = 3;

pub struct Batch2D {
    batch: Batch<Batch2DProvider>,
}

impl Batch2D {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let batch = Batch::new(Batch2DProvider::new(device, queue));
        Batch2D { batch }
    }
}

impl Deref for Batch2D {
    type Target = Batch<Batch2DProvider>;

    fn deref(&self) -> &Self::Target {
        &self.batch
    }
}

impl DerefMut for Batch2D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.batch
    }
}

pub struct Batch2DProvider {
    index_buffer: BatchBuffer<Buffer<u32>>,
    position_buffer: BatchBuffer<Buffer<f32>>,
    texcoord_buffer: BatchBuffer<Buffer<f32>>,
    color_buffer: BatchBuffer<Buffer<f32>>,
}

impl Batch2DProvider {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let index_buffer: BatchBuffer<Buffer<u32>> = BatchBuffer::new(
            device,
            queue,
            &mut None,
            |device, _queue, encoder, prev, len| {
                let label = "IndexBuffer";
                let usage = wgpu::BufferUsage::INDEX
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC;
                match prev {
                    None => {}
                    Some(prev) => {
                        return Buffer::new_with_buffer(
                            device,
                            encoder.as_deref_mut().unwrap(),
                            len,
                            label,
                            usage,
                            prev.0,
                            prev.1,
                        )
                    }
                }
                Buffer::new(device, len, label, usage)
            },
        );
        let position_buffer: BatchBuffer<Buffer<f32>> = BatchBuffer::new(
            device,
            queue,
            &mut None,
            |device, _queue, encoder, prev, len| {
                let label = "PositionBuffer";
                let usage = wgpu::BufferUsage::VERTEX
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC;
                match prev {
                    None => Buffer::new(device, len, label, usage),
                    Some(prev) => Buffer::new_with_buffer(
                        device,
                        encoder.as_deref_mut().unwrap(),
                        len,
                        label,
                        usage,
                        prev.0,
                        prev.1,
                    ),
                }
            },
        );
        let color_buffer: BatchBuffer<Buffer<f32>> = BatchBuffer::new(
            device,
            queue,
            &mut None,
            |device, _queue, encoder, prev, len| {
                let label = "ColorBuffer";
                let usage = wgpu::BufferUsage::VERTEX
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC;
                match prev {
                    None => Buffer::new(device, len, label, usage),
                    Some(prev) => Buffer::new_with_buffer(
                        device,
                        encoder.as_deref_mut().unwrap(),
                        len,
                        label,
                        usage,
                        prev.0,
                        prev.1,
                    ),
                }
            },
        );
        let texcoord_buffer: BatchBuffer<Buffer<f32>> = BatchBuffer::new(
            device,
            queue,
            &mut None,
            |device, _queue, encoder, prev, len| {
                let label = "TexcoordBuffer";
                let usage = wgpu::BufferUsage::VERTEX
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC;
                match prev {
                    None => Buffer::new(device, len, label, usage),
                    Some(prev) => Buffer::new_with_buffer(
                        device,
                        encoder.as_deref_mut().unwrap(),
                        len,
                        label,
                        usage,
                        prev.0,
                        prev.1,
                    ),
                }
            },
        );

        Batch2DProvider {
            index_buffer,
            position_buffer,
            color_buffer,
            texcoord_buffer,
        }
    }

    pub fn index_count(&self) -> usize {
        self.index_buffer.last()
    }

    pub fn index_buffer(&self) -> &Buffer<u32> {
        self.index_buffer.buffer()
    }

    pub fn position_buffer(&self) -> &Buffer<f32> {
        self.position_buffer.buffer()
    }

    pub fn color_buffer(&self) -> &Buffer<f32> {
        self.color_buffer.buffer()
    }

    pub fn texcoord_buffer(&self) -> &Buffer<f32> {
        self.texcoord_buffer.buffer()
    }
}

impl BatchProvider for Batch2DProvider {
    type Device = wgpu::Device;
    type Queue = wgpu::Queue;
    type Encoder = wgpu::CommandEncoder;

    fn run(
        &mut self,
        device: &Self::Device,
        queue: &Self::Queue,
        encoder: &mut Option<&mut Self::Encoder>,
        command: BatchProviderCommand,
    ) {
        match &command {
            BatchProviderCommand::Add { id, data, .. } => {
                debug_assert_eq!(data[1].len(), data[2].len());
                debug_assert_eq!(data[2].len(), data[3].len());

                self.index_buffer.allocate(
                    device,
                    queue,
                    encoder,
                    *id,
                    data[BATCH2D_ATTRIBUTE_INDEX].len(),
                );
                self.position_buffer.allocate(
                    device,
                    queue,
                    encoder,
                    *id,
                    data[BATCH2D_ATTRIBUTE_POSITION].len() * 3,
                );
                self.color_buffer.allocate(
                    device,
                    queue,
                    encoder,
                    *id,
                    data[BATCH2D_ATTRIBUTE_COLOR].len() * 4,
                );
                self.texcoord_buffer.allocate(
                    device,
                    queue,
                    encoder,
                    *id,
                    data[BATCH2D_ATTRIBUTE_TEXCOORD].len() * 2,
                );
            }
            BatchProviderCommand::Remove { id } => {
                self.index_buffer.free(queue, *id);
                self.position_buffer.free(queue, *id);
                self.color_buffer.free(queue, *id);
                self.texcoord_buffer.free(queue, *id);
            }
            BatchProviderCommand::Replace {
                id,
                attribute,
                data,
            } => match *attribute as usize {
                BATCH2D_ATTRIBUTE_INDEX => {
                    self.index_buffer
                        .reallocate(device, queue, encoder, *id, data.len())
                }
                BATCH2D_ATTRIBUTE_POSITION => {
                    self.position_buffer
                        .reallocate(device, queue, encoder, *id, data.len() * 3)
                }
                BATCH2D_ATTRIBUTE_TEXCOORD => {
                    self.texcoord_buffer
                        .reallocate(device, queue, encoder, *id, data.len() * 2)
                }
                BATCH2D_ATTRIBUTE_COLOR => {
                    self.color_buffer
                        .reallocate(device, queue, encoder, *id, data.len() * 4)
                }
                _ => {}
            },
        }
    }

    fn sort(&mut self, ids: Vec<u64>) -> HashSet<u32, RandomState> {
        self.index_buffer.sort(&ids);
        self.position_buffer.sort(&ids);
        self.texcoord_buffer.sort(&ids);
        self.color_buffer.sort(&ids);
        let mut set = HashSet::with_capacity(4);
        set.insert(0);
        set.insert(1);
        set.insert(2);
        set.insert(3);
        set
    }

    fn flush(&mut self, queue: &Self::Queue, batch_object_manager: &mut BatchObjectManager) {
        let index_buffer = &mut self.index_buffer;
        let position_buffer = &mut self.position_buffer;
        let texcoord_buffer = &mut self.texcoord_buffer;
        let color_buffer = &mut self.color_buffer;
        batch_object_manager.flush(|object, attribute| match attribute as usize {
            BATCH2D_ATTRIBUTE_INDEX => {
                let p0 = index_buffer.get_pointer(&object.id()).unwrap();
                let p1 = position_buffer.get_pointer(&object.id()).unwrap();
                let data = object
                    .get_v1u32_data(attribute)
                    .into_iter()
                    .map(|v| *v + p1.first as u32 / 3)
                    .collect::<Vec<_>>();
                index_buffer
                    .buffer()
                    .write(queue, bytemuck::cast_slice(&data), p0.first);
                index_buffer.flush();
            }
            BATCH2D_ATTRIBUTE_POSITION => {
                let p1 = position_buffer.get_pointer(&object.id()).unwrap();
                position_buffer.buffer().write(
                    queue,
                    flatten(object.get_v3f32_data(attribute)),
                    p1.first,
                );
                position_buffer.flush();
            }
            BATCH2D_ATTRIBUTE_TEXCOORD => {
                let p2 = texcoord_buffer.get_pointer(&object.id()).unwrap();
                texcoord_buffer.buffer().write(
                    queue,
                    flatten(object.get_v2f32_data(attribute)),
                    p2.first,
                );
                texcoord_buffer.flush();
            }
            BATCH2D_ATTRIBUTE_COLOR => {
                let p3 = color_buffer.get_pointer(&object.id()).unwrap();
                color_buffer.buffer().write(
                    queue,
                    flatten(object.get_v4f32_data(attribute)),
                    p3.first,
                );
                color_buffer.flush();
            }
            _ => {}
        });
    }
}
