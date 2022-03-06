use bytemuck::Pod;
use std::marker::PhantomData;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, Device, Queue};

pub struct UniformBuffer<T> {
    buffer: Buffer,
    _phantom: PhantomData<T>,
}

impl<T> UniformBuffer<T>
where
    T: Pod,
{
    pub fn new(device: &Device, value: &T) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::bytes_of(value),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        UniformBuffer {
            buffer,
            _phantom: PhantomData,
        }
    }

    pub fn write(&mut self, queue: &Queue, value: &T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
