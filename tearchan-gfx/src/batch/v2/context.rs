use crate::v2::buffer::{BufferCopier, BufferResizer, BufferWriter};

pub struct BatchContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

impl<'a> BatchContext<'a> {
    pub fn writer(&mut self) -> BufferWriter {
        BufferWriter { queue: self.queue }
    }

    pub fn copier(&mut self) -> BufferCopier {
        BufferCopier {
            device: self.device,
            queue: self.queue,
        }
    }

    pub fn resizer(&mut self) -> BufferResizer {
        BufferResizer {
            device: self.device,
            queue: self.queue,
        }
    }
}
