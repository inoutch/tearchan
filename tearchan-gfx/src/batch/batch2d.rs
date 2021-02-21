use crate::batch::buffer::BatchBuffer;
use crate::batch::{Batch, BatchObjectManager, BatchProvider, BatchProviderCommand};
use crate::buffer::{Buffer, BufferInterface};
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use tearchan_util::bytes::flatten;

pub type Batch2D = Batch<Batch2DProvider>;

pub struct Batch2DProvider {
    index_buffer: BatchBuffer<Buffer<u32>>,
    vertex_buffers: Vec<BatchBuffer<Buffer<f32>>>,
}

impl Batch2DProvider {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Batch2D {
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
                            prev,
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
                        prev,
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
                        prev,
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
                        prev,
                    ),
                }
            },
        );

        Batch::new(Batch2DProvider {
            index_buffer,
            vertex_buffers: vec![position_buffer, color_buffer, texcoord_buffer],
        })
    }

    pub fn index_count(&self) -> usize {
        self.index_buffer.last()
    }

    pub fn index_buffer(&self) -> &Buffer<u32> {
        self.index_buffer.buffer()
    }

    pub fn position_buffer(&self) -> &Buffer<f32> {
        self.vertex_buffers[0].buffer()
    }

    pub fn color_buffer(&self) -> &Buffer<f32> {
        self.vertex_buffers[1].buffer()
    }

    pub fn texcoord_buffer(&self) -> &Buffer<f32> {
        self.vertex_buffers[2].buffer()
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

                self.index_buffer
                    .allocate(device, queue, encoder, *id, data[0].len());
                self.vertex_buffers[0].allocate(device, queue, encoder, *id, data[1].len() * 3);
                self.vertex_buffers[1].allocate(device, queue, encoder, *id, data[2].len() * 4);
                self.vertex_buffers[2].allocate(device, queue, encoder, *id, data[3].len() * 2);
            }
            BatchProviderCommand::Remove { id } => {
                self.index_buffer.free(queue, *id);
                self.vertex_buffers[0].free(queue, *id);
                self.vertex_buffers[1].free(queue, *id);
                self.vertex_buffers[2].free(queue, *id);
            }
            BatchProviderCommand::Replace {
                id,
                attribute,
                data,
            } => match attribute {
                0 => self
                    .index_buffer
                    .reallocate(device, queue, encoder, *id, data.len()),
                1 => self.vertex_buffers[0].reallocate(device, queue, encoder, *id, data.len() * 3),
                2 => self.vertex_buffers[1].reallocate(device, queue, encoder, *id, data.len() * 4),
                3 => self.vertex_buffers[2].reallocate(device, queue, encoder, *id, data.len() * 2),
                _ => {}
            },
        }
    }

    fn sort(&mut self, ids: Vec<u64>) -> HashSet<u32, RandomState> {
        self.index_buffer.sort(&ids);
        self.vertex_buffers[0].sort(&ids);
        self.vertex_buffers[1].sort(&ids);
        self.vertex_buffers[2].sort(&ids);
        let mut set = HashSet::with_capacity(4);
        set.insert(0);
        set.insert(1);
        set.insert(2);
        set.insert(3);
        set
    }

    fn flush(&mut self, queue: &Self::Queue, batch_object_manager: &mut BatchObjectManager) {
        let index_buffer = &self.index_buffer;
        let vertex_buffers = &self.vertex_buffers;
        batch_object_manager.flush(|object, attribute| match attribute {
            0 => {
                let p0 = index_buffer.get_pointer(&object.id()).unwrap();
                let p1 = vertex_buffers[0].get_pointer(&object.id()).unwrap();
                let data = object
                    .get_v1u32_data(0)
                    .into_iter()
                    .map(|v| *v + p1.first as u32 / 3)
                    .collect::<Vec<_>>();
                index_buffer
                    .buffer()
                    .write(queue, bytemuck::cast_slice(&data), p0.first);
            }
            1 => {
                let p1 = vertex_buffers[0].get_pointer(&object.id()).unwrap();
                vertex_buffers[0].buffer().write(
                    queue,
                    flatten(object.get_v3f32_data(1)),
                    p1.first,
                );
            }
            2 => {
                let p2 = vertex_buffers[1].get_pointer(&object.id()).unwrap();
                vertex_buffers[1].buffer().write(
                    queue,
                    flatten(object.get_v4f32_data(2)),
                    p2.first,
                );
            }
            3 => {
                let p3 = vertex_buffers[2].get_pointer(&object.id()).unwrap();
                vertex_buffers[2].buffer().write(
                    queue,
                    flatten(object.get_v2f32_data(3)),
                    p3.first,
                );
            }
            _ => {}
        });
    }
}
