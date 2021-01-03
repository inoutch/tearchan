use gfx_hal::adapter::MemoryType;
use gfx_hal::memory::{Properties, Requirements};
use gfx_hal::MemoryTypeId;

pub fn find_memory_type(
    memory_types: &[MemoryType],
    buffer_req: &Requirements,
    properties: Properties,
) -> MemoryTypeId {
    memory_types
        .iter()
        .enumerate()
        .position(|(id, memory_type)| {
            buffer_req.type_mask & (1 << id) as u32 != 0
                && memory_type.properties.contains(properties)
        })
        .unwrap()
        .into()
}
