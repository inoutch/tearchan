use crate::hal::swapchain::SwapchainDescriptor;
use crate::{Adapter, CommandQueue, Device, Surface, SwapchainFrame};
use gfx_hal::format::Format;

pub struct GfxContext<'a> {
    surface: &'a Surface,
    adapters: &'a Vec<Adapter>,
    device: &'a Device,
    queue: &'a CommandQueue,
    swapchain_desc: &'a SwapchainDescriptor,
}

impl<'a> GfxContext<'a> {
    pub fn new(
        surface: &'a Surface,
        adapters: &'a Vec<Adapter>,
        device: &'a Device,
        queue: &'a CommandQueue,
        swapchain_desc: &'a SwapchainDescriptor,
    ) -> GfxContext<'a> {
        GfxContext {
            surface,
            adapters,
            device,
            queue,
            swapchain_desc,
        }
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &CommandQueue {
        &self.queue
    }

    pub fn find_support_format(&self) -> Format {
        self.surface.find_support_format(&self.adapters[0])
    }

    pub fn swapchain_desc(&self) -> &SwapchainDescriptor {
        &self.swapchain_desc
    }
}

pub struct GfxRenderingContext {
    frame: SwapchainFrame,
}

impl GfxRenderingContext {
    pub fn new(frame: SwapchainFrame) -> GfxRenderingContext {
        GfxRenderingContext { frame }
    }

    pub fn frame(&self) -> &SwapchainFrame {
        &self.frame
    }

    pub fn frame_mut(&mut self) -> &mut SwapchainFrame {
        &mut self.frame
    }
}
