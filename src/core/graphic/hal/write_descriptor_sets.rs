use gfx_hal::pso::Descriptor;
use gfx_hal::Backend;

pub struct WriteDescriptorSetsCommon<'a, B: Backend> {
    raw: Vec<gfx_hal::pso::DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>>>,
}

impl<'a, B: Backend> WriteDescriptorSetsCommon<'a, B> {
    pub fn new(
        write_descriptor_sets: Vec<
            gfx_hal::pso::DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>>,
        >,
    ) -> Self {
        WriteDescriptorSetsCommon {
            raw: write_descriptor_sets,
        }
    }

    pub fn get(self) -> Vec<gfx_hal::pso::DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>>> {
        self.raw
    }
}
