use crate::context::{GfxContext, GfxRenderingContext};
use crate::hal::swapchain::{SwapchainCommon, SwapchainDescriptor};
use crate::hal::{DeviceCommon, InstanceCommon, QueueGroupCommon};
use gfx_hal::Features;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Setup<B>
where
    B: gfx_hal::Backend,
{
    window: Window,
    size: PhysicalSize<u32>,
    instance: InstanceCommon<B>,
    device: DeviceCommon<B>,
    queue_group: QueueGroupCommon<B>,
    swapchain: SwapchainCommon<B>,
    swapchain_desc: SwapchainDescriptor,
}

impl<B> Setup<B>
where
    B: gfx_hal::Backend,
{
    pub fn new(window: Window) -> Setup<B> {
        let instance = InstanceCommon::new(&window);
        let size = window.inner_size();

        let surface = instance.surface();
        let adapter = &instance.adapters()[0];
        let family = adapter.find_queue_family();
        let (device, mut queue_groups) =
            adapter.create_device(&[(family, &[1.0])], Features::empty());
        let queue_group = queue_groups.pop().expect("QueueGroup is empty");

        let swapchain_desc = SwapchainDescriptor::new(surface, adapter, &window);
        let swapchain = SwapchainCommon::new(
            &device,
            surface,
            adapter,
            &swapchain_desc,
            queue_group.family(),
        );

        Setup {
            window,
            size,
            instance,
            device,
            queue_group,
            swapchain,
            swapchain_desc,
        }
    }

    pub fn flush(&mut self) {
        self.swapchain.flush();
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
}

impl Setup<crate::hal::backend::Backend> {
    pub fn create_context(&self) -> GfxContext {
        GfxContext::new(
            self.instance.surface(),
            self.instance.adapters(),
            &self.device,
            &self.queue_group,
        )
    }

    pub fn create_render_context(&mut self) -> (GfxContext, GfxRenderingContext) {
        let frame = match self.swapchain.get_current_frame(self.instance.surface()) {
            Ok(frame) => frame,
            Err(_) => {
                self.swapchain = SwapchainCommon::new(
                    &self.device,
                    self.instance.surface(),
                    &self.instance.adapters()[0],
                    &self.swapchain_desc,
                    self.queue_group.family(),
                );
                self.swapchain
                    .get_current_frame(self.instance.surface())
                    .expect("Failed to get current frame")
            }
        };
        (
            GfxContext::new(
                self.instance.surface(),
                self.instance.adapters(),
                &self.device,
                &self.queue_group,
            ),
            GfxRenderingContext::new(frame),
        )
    }
}
