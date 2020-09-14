use gfx_hal::adapter::MemoryType;
use gfx_hal::Limits;

pub struct DeviceInfo {
    pub memory_types: Vec<MemoryType>,
    pub limits: Limits,
}
