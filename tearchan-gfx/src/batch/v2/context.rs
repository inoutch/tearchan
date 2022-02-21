use crate::v2::buffer::{BufferCopier, BufferResizer, BufferWriter};

pub struct BatchContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> BatchContext<'a> {
    pub fn writer(&mut self) -> BufferWriter {
        BufferWriter { queue: self.queue }
    }

    pub fn copier(&mut self) -> BufferCopier {
        BufferCopier {
            encoder: self.encoder,
        }
    }

    pub fn resizer(&mut self) -> BufferResizer {
        BufferResizer {
            device: self.device,
            encoder: self.encoder,
        }
    }
}
