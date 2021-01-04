use crate::hal::{
    AdapterCommon, CommandPoolCommon, DeviceCommon, FenceCommon, SemaphoreCommon, SurfaceCommon,
    TextureCommon,
};
use gfx_hal::format::{Aspects, Format};
use gfx_hal::image::{SubresourceRange, Usage};
use gfx_hal::pool::CommandPoolCreateFlags;
use gfx_hal::queue::QueueFamilyId;
use gfx_hal::window::{AcquireError, Extent2D, PresentationSurface, SwapchainConfig};
use gfx_hal::Backend;
use std::ops::Deref;
use std::rc::Rc;
use winit::window::Window;

pub struct SwapchainFrameCommon<B>
where
    B: Backend,
{
    image: Option<<B::Surface as PresentationSurface<B>>::SwapchainImage>,
    frame_index: usize,
    resource: Rc<SwapchainFrameResource<B>>,
}

impl<B> SwapchainFrameCommon<B>
where
    B: Backend,
{
    pub fn image(&self) -> Option<&<B::Surface as PresentationSurface<B>>::SwapchainImage> {
        self.image.as_ref()
    }

    pub fn pop_image(&mut self) -> Option<<B::Surface as PresentationSurface<B>>::SwapchainImage> {
        std::mem::replace(&mut self.image, None)
    }
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
    pub config: SwapchainConfig,
}

impl SwapchainDescriptor {
    pub fn new<B>(surface: &SurfaceCommon<B>, adapter: &AdapterCommon<B>, window: &Window) -> Self
    where
        B: gfx_hal::Backend,
    {
        let size = window.inner_size();
        let format = surface.find_support_format(adapter);
        let capabilities = surface.capabilities(adapter);
        let config = SwapchainConfig::from_caps(
            &capabilities,
            format,
            Extent2D {
                width: size.width,
                height: size.height,
            },
        );
        SwapchainDescriptor { config }
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
        desc: &SwapchainDescriptor,
        queue_family_id: QueueFamilyId,
    ) -> SwapchainCommon<B> {
        surface
            .configure_swapchain(device, desc.config.clone())
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
                    desc.config.extent.width,
                    desc.config.extent.height,
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
            image: Some(image),
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

    pub fn command_pool(&self) -> &CommandPoolCommon<B> {
        &self.command_pool
    }

    pub fn submission_complete_fence(&self) -> &FenceCommon<B> {
        &self.submission_complete_fence
    }

    pub fn submission_complete_semaphore(&self) -> &SemaphoreCommon<B> {
        &self.submission_complete_semaphore
    }
}
