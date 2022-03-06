use crate::primitive::Primitive;
use nalgebra_glm::{RealField, TVec};
use std::cmp::min;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::RangeBounds;
use tearchan_util::bytes::vec_to_bytes;
use wgpu::util::DeviceExt;
use wgpu::{BufferAddress, BufferSlice, BufferUsages};

pub trait BufferTrait<'a, TDataType> {
    type Resizer: 'a;
    type Writer: 'a;
    type Copier: 'a;

    fn resize(&mut self, resizer: Self::Resizer, len: usize);

    fn write(&mut self, writer: Self::Writer, data: &[TDataType], offset: usize);

    fn copy(&mut self, copy: Self::Copier, from: usize, to: usize, len: usize);

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn clear(&mut self, writer: Self::Writer, offset: usize, len: usize);
}

pub struct Buffer<T> {
    label: String,
    usage: wgpu::BufferUsages,
    buffer: wgpu::Buffer,
    len: usize,
    _t: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new(device: &wgpu::Device, len: usize, label: String, usage: BufferUsages) -> Buffer<T> {
        let u8_len = len * size_of::<T>();
        let bytes: Vec<u8> = vec![0u8; u8_len];
        Buffer::new_with_bytes(device, label, usage, &bytes)
    }

    pub fn new_with_bytes(
        device: &wgpu::Device,
        label: String,
        usage: BufferUsages,
        bytes: &[u8],
    ) -> Buffer<T> {
        assert_eq!(bytes.len() % size_of::<T>(), 0);
        let len = bytes.len() / size_of::<T>();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&label),
            contents: bytes,
            usage,
        });
        Buffer {
            label,
            usage,
            buffer,
            len,
            _t: PhantomData,
        }
    }

    pub fn new_with_buffer(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        len: usize,
        label: String,
        usage: BufferUsages,
        buffer: &Buffer<T>,
        buffer_len: usize,
    ) -> Buffer<T> {
        let copy_len = min(len, min(buffer.len, buffer_len)) * size_of::<T>();
        let new_buffer = Self::new(device, len, label, usage);
        encoder.copy_buffer_to_buffer(&buffer.buffer, 0, &new_buffer.buffer, 0, copy_len as u64);
        new_buffer
    }

    #[inline]
    pub fn slice<S: RangeBounds<BufferAddress>>(&self, bounds: S) -> BufferSlice {
        self.buffer.slice(bounds)
    }
}

pub struct BufferResizer<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

pub struct BufferWriter<'a> {
    pub queue: &'a wgpu::Queue,
}

pub struct BufferCopier<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

impl<'a, T: Primitive> BufferTrait<'a, T> for Buffer<T> {
    type Resizer = BufferResizer<'a>;
    type Writer = BufferWriter<'a>;
    type Copier = BufferCopier<'a>;

    fn resize(&mut self, resizer: BufferResizer<'a>, len: usize) {
        let u8_len = len * size_of::<T>();
        let bytes: Vec<u8> = vec![0u8; u8_len];
        let buffer = resizer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&self.label),
                contents: &bytes,
                usage: self.usage,
            });

        let copy_u8_len = min(len, self.len) * size_of::<T>();
        let mut encoder = resizer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &buffer, 0, copy_u8_len as u64);
        resizer.queue.submit(Some(encoder.finish()));
        self.buffer = buffer;
        self.len = len;
    }

    fn write(&mut self, writer: BufferWriter<'a>, data: &[T], offset: usize) {
        let u8_offset = offset * size_of::<T>();
        writer
            .queue
            .write_buffer(&self.buffer, u8_offset as u64, bytemuck::cast_slice(data));
    }

    fn copy(&mut self, copy: BufferCopier, from: usize, to: usize, len: usize) {
        let from_u8_offset = from * size_of::<T>();
        let to_u8_offset = to * size_of::<T>();
        let u8_len = len * size_of::<T>();
        let mut encoder = copy
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(
            &self.buffer,
            from_u8_offset as u64,
            &self.buffer,
            to_u8_offset as u64,
            u8_len as u64,
        );
        copy.queue.submit(Some(encoder.finish()));
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn clear(&mut self, writer: BufferWriter<'a>, offset: usize, len: usize) {
        let u8_offset = offset * size_of::<T>() as usize;
        let u8_size = len * size_of::<T>() as usize;

        let bytes: Vec<u8> = vec![0u8; u8_size];
        writer
            .queue
            .write_buffer(&self.buffer, u8_offset as u64, &bytes);
    }
}

impl<'a, T: RealField, const D: usize> BufferTrait<'a, TVec<T, D>> for Buffer<TVec<T, D>> {
    type Resizer = BufferResizer<'a>;
    type Writer = BufferWriter<'a>;
    type Copier = BufferCopier<'a>;

    fn resize(&mut self, resizer: BufferResizer<'a>, len: usize) {
        let u8_len = len * size_of::<TVec<T, D>>();
        let bytes: Vec<u8> = vec![0u8; u8_len];
        let buffer = resizer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&self.label),
                contents: &bytes,
                usage: self.usage,
            });

        let copy_u8_len = min(len, self.len) * size_of::<TVec<T, D>>();
        let mut encoder = resizer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &buffer, 0, copy_u8_len as u64);
        resizer.queue.submit(Some(encoder.finish()));
        self.buffer = buffer;
        self.len = len;
    }

    fn write(&mut self, writer: BufferWriter, data: &[TVec<T, D>], offset: usize) {
        let u8_offset = offset * size_of::<TVec<T, D>>();
        writer
            .queue
            .write_buffer(&self.buffer, u8_offset as u64, vec_to_bytes(data));
    }

    fn copy(&mut self, copy: BufferCopier, from: usize, to: usize, len: usize) {
        let from_u8_offset = from * size_of::<TVec<T, D>>();
        let to_u8_offset = to * size_of::<TVec<T, D>>();
        let u8_len = len * size_of::<TVec<T, D>>();
        let mut encoder = copy
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(
            &self.buffer,
            from_u8_offset as u64,
            &self.buffer,
            to_u8_offset as u64,
            u8_len as u64,
        );
        copy.queue.submit(Some(encoder.finish()));
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn clear(&mut self, writer: BufferWriter, offset: usize, len: usize) {
        let u8_offset = offset * size_of::<TVec<T, D>>() as usize;
        let u8_size = len * size_of::<TVec<T, D>>() as usize;

        let bytes: Vec<u8> = vec![0u8; u8_size];
        writer
            .queue
            .write_buffer(&self.buffer, u8_offset as u64, &bytes);
    }
}

#[cfg(test)]
pub mod test {
    use crate::v2::buffer::BufferTrait;
    use std::cell::RefCell;

    pub struct TestWriter;
    pub struct TestResizer;
    pub struct TestCopier;

    pub struct TestBuffer<T> {
        pub data: RefCell<Vec<T>>,
    }

    impl<T> TestBuffer<T> {
        pub fn new(data: Vec<T>) -> TestBuffer<T> {
            Self {
                data: RefCell::new(data),
            }
        }
    }

    impl<'a, T> BufferTrait<'a, T> for TestBuffer<T>
    where
        T: Default + Copy,
    {
        type Resizer = &'a TestResizer;
        type Writer = &'a TestWriter;
        type Copier = &'a TestCopier;

        fn resize(&mut self, _resizer: Self::Resizer, len: usize) {
            self.data.borrow_mut().resize(len, T::default());
        }

        fn write(&mut self, _writer: Self::Writer, data: &[T], offset: usize) {
            self.data
                .borrow_mut()
                .splice(offset..(offset + data.len()), data.iter().copied());
        }

        fn copy(&mut self, _copy: Self::Copier, from: usize, to: usize, len: usize) {
            let from = {
                self.data.borrow_mut().as_slice()[from..(from + len)]
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
            };
            self.data.borrow_mut().splice(to..(to + len), from);
        }

        fn len(&self) -> usize {
            self.data.borrow().len()
        }

        fn is_empty(&self) -> bool {
            self.data.borrow().is_empty()
        }

        fn clear(&mut self, _writer: Self::Writer, offset: usize, len: usize) {
            self.data
                .borrow_mut()
                .splice(offset..(offset + len), vec![T::default(); len]);
        }
    }
}
