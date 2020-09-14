use gfx_hal::adapter::Adapter;
use gfx_hal::queue::QueueFamily;
use gfx_hal::window::Surface;
use gfx_hal::Backend;

pub fn find_queue_family<'a, B: Backend>(
    adapter: &'a Adapter<B>,
    surface: &B::Surface,
) -> &'a B::QueueFamily {
    adapter
        .queue_families
        .iter()
        .find(|family| {
            surface.supports_queue_family(family) && family.queue_type().supports_graphics()
        })
        .unwrap()
}
