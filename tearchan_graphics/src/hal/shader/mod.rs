use crate::hal::render_bundle::RenderBundleCommon;
use crate::hal::shader::attribute::Attribute;
use crate::hal::shader::shader_source::ShaderSource;
use gfx_hal::device::Device;
use gfx_hal::pso::{DescriptorSetLayoutBinding, EntryPoint, Specialization};
use gfx_hal::Backend;
use std::mem::ManuallyDrop;
use std::ops::Deref;

pub mod attribute;
pub mod descriptor_set;
pub mod shader_source;
pub mod write_descriptor_sets;

const ENTRY_NAME: &str = "main";

pub struct ShaderCommon<B: Backend> {
    render_bundle: RenderBundleCommon<B>,
    vs_module: ManuallyDrop<B::ShaderModule>,
    fs_module: ManuallyDrop<B::ShaderModule>,
    attributes: Vec<Attribute>,
    descriptor_set_layout_bindings: Vec<gfx_hal::pso::DescriptorSetLayoutBinding>,
}

impl<B: Backend> ShaderCommon<B> {
    pub fn new(
        render_bundle: &RenderBundleCommon<B>,
        shader_source: ShaderSource,
        attributes: Vec<Attribute>,
        descriptor_sets: Vec<gfx_hal::pso::DescriptorSetLayoutBinding>,
    ) -> ShaderCommon<B> {
        let vs_module = unsafe {
            render_bundle
                .device()
                .create_shader_module(&shader_source.spirv_vert_source)
        }
        .unwrap();
        let fs_module = unsafe {
            render_bundle
                .device()
                .create_shader_module(&shader_source.spirv_frag_source)
        }
        .unwrap();

        ShaderCommon {
            render_bundle: render_bundle.clone(),
            vs_module: ManuallyDrop::new(vs_module),
            fs_module: ManuallyDrop::new(fs_module),
            attributes,
            descriptor_set_layout_bindings: descriptor_sets,
        }
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    pub fn descriptor_set_layout_bindings(&self) -> &Vec<DescriptorSetLayoutBinding> {
        &self.descriptor_set_layout_bindings
    }

    pub fn vs_entry(&self) -> EntryPoint<B> {
        EntryPoint {
            entry: ENTRY_NAME,
            module: self.vs_module.deref(),
            specialization: Specialization::default(),
        }
    }

    pub fn fs_entry(&self) -> EntryPoint<B> {
        EntryPoint {
            entry: ENTRY_NAME,
            module: self.fs_module.deref(),
            specialization: Specialization::default(),
        }
    }
}

impl<B: Backend> Drop for ShaderCommon<B> {
    fn drop(&mut self) {
        unsafe {
            self.render_bundle
                .device()
                .destroy_shader_module(ManuallyDrop::into_inner(std::ptr::read(&self.vs_module)));
            self.render_bundle
                .device()
                .destroy_shader_module(ManuallyDrop::into_inner(std::ptr::read(&self.fs_module)));
        }
    }
}
