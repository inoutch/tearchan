use crate::core::graphic::shader::attribute::Attribute;
use crate::core::graphic::shader::shader_source::ShaderSource;
use gfx_hal::device::Device;
use gfx_hal::Backend;
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

pub struct Shader<B: Backend> {
    device: Weak<B::Device>,
    vs_module: ManuallyDrop<B::ShaderModule>,
    fs_module: ManuallyDrop<B::ShaderModule>,
    attributes: Vec<Attribute>,
}

impl<B: Backend> Shader<B> {
    pub fn new(
        device: &Rc<B::Device>,
        shader_source: ShaderSource,
        attributes: Vec<Attribute>,
    ) -> Shader<B> {
        let vs_module =
            unsafe { device.create_shader_module(&shader_source.spirv_vert_source) }.unwrap();
        let fs_module =
            unsafe { device.create_shader_module(&shader_source.spirv_frag_source) }.unwrap();
        Shader {
            device: Rc::downgrade(device),
            vs_module: ManuallyDrop::new(vs_module),
            fs_module: ManuallyDrop::new(fs_module),
            attributes,
        }
    }
}

impl<B: Backend> Drop for Shader<B> {
    fn drop(&mut self) {
        if let Some(x) = self.device.upgrade() {
            unsafe {
                x.destroy_shader_module(ManuallyDrop::into_inner(std::ptr::read(&self.vs_module)));
                x.destroy_shader_module(ManuallyDrop::into_inner(std::ptr::read(&self.fs_module)));
            }
        }
    }
}
