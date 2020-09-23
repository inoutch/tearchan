use crate::hal::render_bundle::RenderBundleCommon;
use gfx_hal::device::Device;
use gfx_hal::pso::{Descriptor, DescriptorSetWrite};
use gfx_hal::Backend;

#[derive(Debug)]
pub struct WriteDescriptorSetsCommon<'a, B: Backend> {
    raw: Vec<DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>>>,
}

impl<'a, B: Backend> WriteDescriptorSetsCommon<'a, B> {
    pub fn new(
        write_descriptor_sets: Vec<DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>>>,
    ) -> Self {
        WriteDescriptorSetsCommon {
            raw: write_descriptor_sets,
        }
    }

    pub fn get(self) -> Vec<DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>>> {
        self.raw
    }

    pub fn write(self, render_bundle: &RenderBundleCommon<B>) {
        unsafe { render_bundle.device().write_descriptor_sets(self.get()) }
    }
}
