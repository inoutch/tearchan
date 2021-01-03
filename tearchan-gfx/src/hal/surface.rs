use gfx_hal::adapter::Adapter;
use gfx_hal::format::{ChannelType, Format};
use gfx_hal::window::Surface;
use gfx_hal::Backend;

pub fn find_support_format<B: Backend>(surface: &B::Surface, adapter: &Adapter<B>) -> Format {
    let formats = surface.supported_formats(&adapter.physical_device);
    formats.map_or(Format::Rgba8Srgb, |formats| {
        formats
            .iter()
            .find(|format| format.base_format().1 == ChannelType::Srgb)
            .copied()
            .unwrap_or(formats[0])
    })
}
