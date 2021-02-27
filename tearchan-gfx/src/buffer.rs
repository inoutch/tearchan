use bytemuck::Pod;
use std::cmp::min;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::RangeBounds;
use wgpu::util::DeviceExt;
use wgpu::{BufferAddress, BufferSlice, BufferUsage};

pub trait BufferInterface {
    type DataType: Pod;
    type Device;
    type Queue;
    type Encoder;

    fn write(&self, queue: &Self::Queue, data: &[Self::DataType], offset: usize);

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn clear(&self, queue: &Self::Queue, offset: usize, len: usize);
}

pub struct Buffer<T> {
    buffer: wgpu::Buffer,
    len: usize,
    _type: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new(device: &wgpu::Device, len: usize, label: &str, usage: BufferUsage) -> Buffer<T> {
        let u8_len = len * size_of::<T>();
        let mut bytes: Vec<u8> = vec![];
        for _ in 0..u8_len {
            bytes.push(0u8);
        }

        Buffer::new_with_bytes(device, label, usage, &bytes)
    }

    pub fn new_with_bytes(
        device: &wgpu::Device,
        label: &str,
        usage: BufferUsage,
        bytes: &[u8],
    ) -> Buffer<T> {
        assert_eq!(bytes.len() % size_of::<T>(), 0);
        let len = bytes.len() / size_of::<T>();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &bytes,
            usage,
        });
        Buffer {
            buffer,
            len,
            _type: PhantomData,
        }
    }

    pub fn new_with_buffer(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        len: usize,
        label: &str,
        usage: BufferUsage,
        buffer: &Buffer<T>,
        buffer_len: usize,
    ) -> Buffer<T> {
        let copy_len = min(len, min(buffer.len, buffer_len)) * size_of::<T>();
        let new_buffer = Self::new(device, len, label, usage);
        encoder.copy_buffer_to_buffer(&buffer.buffer, 0, &new_buffer.buffer, 0, copy_len as u64);
        new_buffer
    }

    pub fn slice<S: RangeBounds<BufferAddress>>(&self, bounds: S) -> BufferSlice {
        self.buffer.slice(bounds)
    }
}

impl<T> BufferInterface for Buffer<T>
where
    T: Pod,
{
    type DataType = T;
    type Device = wgpu::Device;
    type Queue = wgpu::Queue;
    type Encoder = wgpu::CommandEncoder;

    fn write(&self, queue: &Self::Queue, data: &[T], offset: usize) {
        let u8_offset = offset * size_of::<T>();
        queue.write_buffer(&self.buffer, u8_offset as u64, bytemuck::cast_slice(data));
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn clear(&self, queue: &Self::Queue, offset: usize, len: usize) {
        let u8_offset = offset * size_of::<T>() as usize;
        let u8_size = len * size_of::<T>() as usize;

        let mut bytes: Vec<u8> = vec![];
        for _ in 0..u8_size {
            bytes.push(0u8);
        }

        queue.write_buffer(&self.buffer, u8_offset as u64, &bytes);
    }
}
