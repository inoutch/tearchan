use crate::batch::buffer::BatchBuffer;
use crate::batch::{Batch, BatchObjectManager, BatchProvider, BatchProviderCommand};
use crate::buffer::{Buffer, BufferInterface};
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use tearchan_util::bytes::flatten;

pub const BATCH3D_ATTRIBUTE_INDEX: usize = 0;
pub const BATCH3D_ATTRIBUTE_POSITION: usize = 1;
pub const BATCH3D_ATTRIBUTE_TEXCOORD: usize = 2;
pub const BATCH3D_ATTRIBUTE_COLOR: usize = 3;
pub const BATCH3D_ATTRIBUTE_NORMAL: usize = 4;

pub struct Batch3D {
    batch: Batch<Batch3DProvider>,
}

impl Batch3D {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let batch = Batch::new(Batch3DProvider::new(device, queue));
        Batch3D { batch }
    }
}

impl Deref for Batch3D {
    type Target = Batch<Batch3DProvider>;

    fn deref(&self) -> &Self::Target {
        &self.batch
    }
}

impl DerefMut for Batch3D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.batch
    }
}

pub struct Batch3DProvider {
    index_buffer: BatchBuffer<Buffer<u32>>,
    position_buffer: BatchBuffer<Buffer<f32>>,
    texcoord_buffer: BatchBuffer<Buffer<f32>>,
    color_buffer: BatchBuffer<Buffer<f32>>,
    normal_buffer: BatchBuffer<Buffer<f32>>,
}

impl Batch3DProvider {
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
        let normal_buffer: BatchBuffer<Buffer<f32>> = BatchBuffer::new(
            device,
            queue,
            &mut None,
            |device, _queue, encoder, prev, len| {
                let label = "NormalBuffer";
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

        Batch3DProvider {
            index_buffer,
            position_buffer,
            texcoord_buffer,
            color_buffer,
            normal_buffer,
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

    pub fn texcoord_buffer(&self) -> &Buffer<f32> {
        self.texcoord_buffer.buffer()
    }

    pub fn color_buffer(&self) -> &Buffer<f32> {
        self.color_buffer.buffer()
    }

    pub fn normal_buffer(&self) -> &Buffer<f32> {
        self.normal_buffer.buffer()
    }
}

impl BatchProvider for Batch3DProvider {
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
                debug_assert_eq!(data[3].len(), data[4].len());

                self.index_buffer.allocate(
                    device,
                    queue,
                    encoder,
                    *id,
                    data[BATCH3D_ATTRIBUTE_INDEX].len(),
                );
                self.position_buffer.allocate(
                    device,
                    queue,
                    encoder,
                    *id,
                    data[BATCH3D_ATTRIBUTE_POSITION].len() * 3,
                );
                self.texcoord_buffer.allocate(
                    device,
                    queue,
                    encoder,
                    *id,
                    data[BATCH3D_ATTRIBUTE_TEXCOORD].len() * 2,
                );
                self.color_buffer.allocate(
                    device,
                    queue,
                    encoder,
                    *id,
                    data[BATCH3D_ATTRIBUTE_COLOR].len() * 4,
                );
                self.normal_buffer.allocate(
                    device,
                    queue,
                    encoder,
                    *id,
                    data[BATCH3D_ATTRIBUTE_NORMAL].len() * 3,
                );
            }
            BatchProviderCommand::Remove { id } => {
                self.index_buffer.free(queue, *id);
                self.position_buffer.free(queue, *id);
                self.texcoord_buffer.free(queue, *id);
                self.color_buffer.free(queue, *id);
                self.normal_buffer.free(queue, *id);
            }
            BatchProviderCommand::Replace {
                id,
                attribute,
                data,
            } => match *attribute as usize {
                BATCH3D_ATTRIBUTE_INDEX => {
                    self.index_buffer
                        .reallocate(device, queue, encoder, *id, data.len())
                }
                BATCH3D_ATTRIBUTE_POSITION => {
                    self.position_buffer
                        .reallocate(device, queue, encoder, *id, data.len() * 3)
                }
                BATCH3D_ATTRIBUTE_TEXCOORD => {
                    self.texcoord_buffer
                        .reallocate(device, queue, encoder, *id, data.len() * 2)
                }
                BATCH3D_ATTRIBUTE_COLOR => {
                    self.color_buffer
                        .reallocate(device, queue, encoder, *id, data.len() * 4)
                }
                BATCH3D_ATTRIBUTE_NORMAL => {
                    self.normal_buffer
                        .reallocate(device, queue, encoder, *id, data.len() * 3)
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
        self.normal_buffer.sort(&ids);
        let mut set = HashSet::with_capacity(5);
        set.insert(BATCH3D_ATTRIBUTE_INDEX as u32);
        set.insert(BATCH3D_ATTRIBUTE_POSITION as u32);
        set.insert(BATCH3D_ATTRIBUTE_TEXCOORD as u32);
        set.insert(BATCH3D_ATTRIBUTE_COLOR as u32);
        set.insert(BATCH3D_ATTRIBUTE_NORMAL as u32);
        set
    }

    fn flush(&mut self, queue: &Self::Queue, manager: &mut BatchObjectManager) {
        let index_buffer = &mut self.index_buffer;
        let position_buffer = &mut self.position_buffer;
        let texcoord_buffer = &mut self.texcoord_buffer;
        let color_buffer = &mut self.color_buffer;
        let normal_buffer = &mut self.normal_buffer;
        manager.flush(|object, attribute| match attribute as usize {
            BATCH3D_ATTRIBUTE_INDEX => {
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
            BATCH3D_ATTRIBUTE_POSITION => {
                let p1 = position_buffer.get_pointer(&object.id()).unwrap();
                position_buffer.buffer().write(
                    queue,
                    flatten(object.get_v3f32_data(attribute)),
                    p1.first,
                );
                position_buffer.flush();
            }
            BATCH3D_ATTRIBUTE_TEXCOORD => {
                let p2 = texcoord_buffer.get_pointer(&object.id()).unwrap();
                texcoord_buffer.buffer().write(
                    queue,
                    flatten(object.get_v2f32_data(attribute)),
                    p2.first,
                );
                texcoord_buffer.flush();
            }
            BATCH3D_ATTRIBUTE_COLOR => {
                let p3 = color_buffer.get_pointer(&object.id()).unwrap();
                color_buffer.buffer().write(
                    queue,
                    flatten(object.get_v4f32_data(attribute)),
                    p3.first,
                );
                color_buffer.flush();
            }
            BATCH3D_ATTRIBUTE_NORMAL => {
                let p4 = normal_buffer.get_pointer(&object.id()).unwrap();
                normal_buffer.buffer().write(
                    queue,
                    flatten(object.get_v3f32_data(attribute)),
                    p4.first,
                );
                normal_buffer.flush();
            }
            _ => {}
        });
    }
}
