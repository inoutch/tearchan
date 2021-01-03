use crate::hal::{
    AdapterCommon, CommandPoolCommon, DeviceCommon, FenceCommon, SemaphoreCommon, SurfaceCommon,
    TextureCommon,
};
use gfx_hal::format::{Aspects, Format};
use gfx_hal::image::{SubresourceRange, Usage};
use gfx_hal::pool::CommandPoolCreateFlags;
use gfx_hal::queue::QueueFamilyId;
use gfx_hal::window::{AcquireError, Extent2D, SwapchainConfig};
use gfx_hal::Backend;
use std::borrow::Borrow;
use std::ops::Deref;
use std::rc::Rc;
use winit::window::Window;

pub struct SwapchainFrameCommon<B>
where
    B: Backend,
{
    image: Box<dyn Borrow<B::ImageView>>,
    frame_index: usize,
    resource: Rc<SwapchainFrameResource<B>>,
}

impl<B> Deref for SwapchainFrameCommon<B>
where
    B: Backend,
{
    type Target = SwapchainFrameResource<B>;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

pub struct SwapchainDescriptor {
    pub format: Format,
    pub width: u32,
    pub height: u32,
}

impl SwapchainDescriptor {
    pub fn new<B>(surface: &SurfaceCommon<B>, adapter: &AdapterCommon<B>, window: &Window) -> Self
    where
        B: gfx_hal::Backend,
    {
        let size = window.inner_size();
        let format = surface.find_support_format(adapter);
        SwapchainDescriptor {
            format,
            width: size.width,
            height: size.height,
        }
    }
}

pub struct SwapchainCommon<B>
where
    B: Backend,
{
    frame_resources: Vec<Rc<SwapchainFrameResource<B>>>,
    frame_index: usize,
}

impl<B> SwapchainCommon<B>
where
    B: Backend,
{
    pub fn new(
        device: &DeviceCommon<B>,
        surface: &SurfaceCommon<B>,
        adapter: &AdapterCommon<B>,
        desc: &SwapchainDescriptor,
        queue_family_id: QueueFamilyId,
    ) -> SwapchainCommon<B> {
        let capabilities = surface.capabilities(adapter);
        let swap_config = SwapchainConfig::from_caps(
            &capabilities,
            desc.format,
            Extent2D {
                width: desc.width,
                height: desc.height,
            },
        );
        surface
            .configure_swapchain(device, swap_config)
            .expect("Can't configure swapchain");

        let frame_in_flight = 3;
        let mut frame_resources = Vec::with_capacity(frame_in_flight);
        for _ in 0..frame_in_flight {
            frame_resources.push(Rc::new(SwapchainFrameResource {
                command_pool: device
                    .create_command_pool(queue_family_id, CommandPoolCreateFlags::RESET_INDIVIDUAL),
                submission_complete_fence: device.create_fence(true),
                submission_complete_semaphore: device.create_semaphore(),
                depth_texture: device.create_texture(
                    Format::D32Sfloat,
                    Usage::DEPTH_STENCIL_ATTACHMENT,
                    SubresourceRange {
                        aspects: Aspects::DEPTH,
                        ..SubresourceRange::default()
                    },
                    desc.width,
                    desc.height,
                ),
            }));
        }

        SwapchainCommon {
            frame_resources,
            frame_index: 0,
        }
    }

    pub fn get_current_frame(
        &self,
        surface: &SurfaceCommon<B>,
    ) -> Result<SwapchainFrameCommon<B>, AcquireError> {
        let (image, _) = surface.acquire_image(!0)?;
        Ok(SwapchainFrameCommon {
            image: Box::new(image),
            frame_index: self.frame_index,
            resource: Rc::clone(&self.frame_resources[self.frame_index]),
        })
    }

    pub fn flush(&mut self) {
        self.frame_index += 1;
        self.frame_index = self.frame_resources.len() % self.frame_index;
    }
}

pub struct SwapchainFrameResource<B: Backend> {
    command_pool: CommandPoolCommon<B>,
    submission_complete_fence: FenceCommon<B>, // Used in synchronization with CPU
    submission_complete_semaphore: SemaphoreCommon<B>, // Used in synchronization with Queue
    depth_texture: TextureCommon<B>,
}

impl<B: Backend> SwapchainFrameResource<B>
where
    B: Backend,
{
    pub fn depth_texture(&self) -> &TextureCommon<B> {
        &self.depth_texture
    }
}
