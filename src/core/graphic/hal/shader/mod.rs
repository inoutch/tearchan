use crate::core::graphic::hal::shader::attribute::Attribute;
use crate::core::graphic::hal::shader::shader_source::ShaderSource;
use gfx_hal::device::Device;
use gfx_hal::Backend;
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

pub mod attribute;
pub mod shader_source;

const ENTRY_NAME: &str = "main";

pub struct ShaderCommon<B: Backend> {
    device: Weak<B::Device>,
    vs_module: ManuallyDrop<B::ShaderModule>,
    fs_module: ManuallyDrop<B::ShaderModule>,
    attributes: Vec<Attribute>,
    descriptor_set_layout_bindings: Vec<gfx_hal::pso::DescriptorSetLayoutBinding>,
}

impl<B: Backend> ShaderCommon<B> {
    pub fn new(
        device: &Rc<B::Device>,
        shader_source: ShaderSource,
        attributes: Vec<Attribute>,
        descriptor_sets: Vec<gfx_hal::pso::DescriptorSetLayoutBinding>,
    ) -> ShaderCommon<B> {
        let vs_module =
            unsafe { device.create_shader_module(&shader_source.spirv_vert_source) }.unwrap();
        let fs_module =
            unsafe { device.create_shader_module(&shader_source.spirv_frag_source) }.unwrap();

        ShaderCommon {
            device: Rc::downgrade(device),
            vs_module: ManuallyDrop::new(vs_module),
            fs_module: ManuallyDrop::new(fs_module),
            attributes,
            descriptor_set_layout_bindings: descriptor_sets,
        }
    }

    pub fn borrow_attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    pub fn borrow_descriptor_set_layout_bindings(
        &self,
    ) -> &Vec<gfx_hal::pso::DescriptorSetLayoutBinding> {
        &self.descriptor_set_layout_bindings
    }

    pub fn create_entries(&self) -> gfx_hal::pso::GraphicsShaderSet<B> {
        gfx_hal::pso::GraphicsShaderSet {
            vertex: gfx_hal::pso::EntryPoint::<B> {
                entry: ENTRY_NAME,
                module: &self.vs_module,
                specialization: gfx_hal::pso::Specialization::default(),
            },
            hull: None,
            domain: None,
            geometry: None,
            fragment: Some(gfx_hal::pso::EntryPoint::<B> {
                entry: ENTRY_NAME,
                module: &self.fs_module,
                specialization: gfx_hal::pso::Specialization::default(),
            }),
        }
    }
}

impl<B: Backend> Drop for ShaderCommon<B> {
    fn drop(&mut self) {
        if let Some(x) = self.device.upgrade() {
            unsafe {
                x.destroy_shader_module(ManuallyDrop::into_inner(std::ptr::read(&self.vs_module)));
                x.destroy_shader_module(ManuallyDrop::into_inner(std::ptr::read(&self.fs_module)));
            }
        }
    }
}
