use gfx_hal::Backend;

pub struct DescriptorSetCommon<B: Backend> {
    raw: B::DescriptorSet,
}

impl<B: Backend> DescriptorSetCommon<B> {
    pub fn new(descriptor_set: B::DescriptorSet) -> Self {
        DescriptorSetCommon {
            raw: descriptor_set,
        }
    }

    pub fn get(&self) -> &B::DescriptorSet {
        &self.raw
    }
}
