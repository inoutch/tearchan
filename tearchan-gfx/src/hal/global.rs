use gfx_hal::adapter::MemoryProperties;
use gfx_hal::command::{ClearValue, CommandBufferFlags, Level, SubpassContents};
use gfx_hal::device::{OutOfMemory, WaitError};
use gfx_hal::format::{ChannelType, Format};
use gfx_hal::image::Extent;
use gfx_hal::memory::{Requirements, Segment};
use gfx_hal::pool::CommandPoolCreateFlags;
use gfx_hal::queue::{QueueFamily, QueueFamilyId, QueuePriority, Submission};
use gfx_hal::window::{
    AcquireError, PresentError, PresentationSurface, Suboptimal, SurfaceCapabilities,
    SwapchainConfig, SwapchainError,
};
use gfx_hal::{pass, pso, Backend, Features, Limits, MemoryTypeId};
use std::borrow::Borrow;

pub type AdapterId = u64;
pub type SurfaceId = u64;
pub type DeviceId = u64;
pub type CommandPoolId = u64;
pub type CommandBufferId = u64;
pub type SemaphoreId = u64;
pub type FenceId = u64;
pub type ImageId = u64;
pub type ImageViewId = u64;
pub type MemoryId = u64;
pub type QueueGroupId = u64;
pub type ShaderModuleId = u64;
pub type BufferId = u64;
pub type MemoryMapId = u64;
pub type RenderPassId = u64;
pub type FramebufferId = u64;

pub struct Instance<B>
where
    B: Backend,
{
    pub raw: B::Instance,
}

impl<B> Instance<B>
where
    B: Backend,
{
    pub fn destroy_surface(&self, surface: Surface<B>) {
        use gfx_hal::Instance;
        unsafe { self.raw.destroy_surface(surface.raw) };
    }
}

pub struct Adapter<B>
where
    B: Backend,
{
    pub raw: gfx_hal::adapter::Adapter<B>,
    pub id: AdapterId,
}

impl<B> Adapter<B>
where
    B: Backend,
{
    pub fn find_queue_family(&self, surface: &Surface<B>) -> QueueFamilyId {
        use gfx_hal::window::Surface;
        self.raw
            .queue_families
            .iter()
            .find(|family| {
                surface.raw.supports_queue_family(family) && family.queue_type().supports_graphics()
            })
            .unwrap()
            .id()
    }

    pub fn create_device(
        &self,
        device_id: DeviceId,
        families: &[(QueueFamilyId, &[QueuePriority])],
        requested_features: Features,
    ) -> (Device<B>, Vec<gfx_hal::queue::QueueGroup<B>>) {
        use gfx_hal::adapter::PhysicalDevice;

        let families = families
            .iter()
            .map(|(queue_family_id, priority)| {
                (
                    self.raw
                        .queue_families
                        .iter()
                        .find(|family| family.id() == *queue_family_id)
                        .expect("Not found queue family"),
                    *priority,
                )
            })
            .collect::<Vec<_>>();
        let gpu = unsafe {
            self.raw
                .physical_device
                .open(families.as_slice(), requested_features)
        }
        .expect("Failed to open device");

        (
            Device {
                raw: gpu.device,
                id: device_id,
                adapter_id: self.id,
                limits: self.raw.physical_device.limits(),
                memory_properties: self.raw.physical_device.memory_properties(),
            },
            gpu.queue_groups,
        )
    }
}

pub struct Surface<B>
where
    B: Backend,
{
    pub raw: B::Surface,
    pub id: SurfaceId,
}

impl<B> Surface<B>
where
    B: Backend,
{
    pub fn capabilities(&self, adapter: &Adapter<B>) -> SurfaceCapabilities {
        use gfx_hal::window::Surface;
        self.raw.capabilities(&adapter.raw.physical_device)
    }

    pub fn find_support_format(&self, adapter: &Adapter<B>) -> Format {
        use gfx_hal::window::Surface;
        let formats = self.raw.supported_formats(&adapter.raw.physical_device);
        formats.map_or(Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .copied()
                .unwrap_or(formats[0])
        })
    }

    pub fn configure_swapchain(
        &mut self,
        device: &Device<B>,
        swap_config: SwapchainConfig,
    ) -> Result<(), SwapchainError> {
        unsafe { self.raw.configure_swapchain(&device.raw, swap_config) }
    }

    pub fn acquire_image(
        &mut self,
        timeout_ns: u64,
    ) -> Result<
        (
            <<B as Backend>::Surface as PresentationSurface<B>>::SwapchainImage,
            Option<Suboptimal>,
        ),
        AcquireError,
    > {
        unsafe { self.raw.acquire_image(timeout_ns) }
    }
}

pub struct Device<B>
where
    B: Backend,
{
    pub raw: B::Device,
    pub id: DeviceId,
    pub adapter_id: AdapterId,
    pub limits: Limits,
    pub memory_properties: MemoryProperties,
}

impl<B> Device<B>
where
    B: Backend,
{
    pub fn get_image_requirements(&self, image: &Image<B>) -> Requirements {
        use gfx_hal::device::Device;
        unsafe { self.raw.get_image_requirements(&image.raw) }
    }

    pub fn bind_image_memory(&self, memory: &Memory<B>, offset: u64, image: &mut Image<B>) {
        use gfx_hal::device::Device;
        unsafe {
            self.raw
                .bind_image_memory(&memory.raw, offset, &mut image.raw)
        }
        .expect("Failed to bind ImageMemory");
    }

    pub fn create_command_pool(
        &self,
        id: CommandPoolId,
        family: QueueFamilyId,
        create_flags: CommandPoolCreateFlags,
    ) -> CommandPool<B> {
        use gfx_hal::device::Device;
        let command_pool = unsafe { self.raw.create_command_pool(family, create_flags) }
            .expect("Failed to create command pool");
        CommandPool {
            raw: command_pool,
            id,
            device_id: self.id,
        }
    }

    pub fn create_semaphore(&self, id: SemaphoreId) -> Semaphore<B> {
        use gfx_hal::device::Device;
        let semaphore = self
            .raw
            .create_semaphore()
            .expect("Failed to create semaphore");
        Semaphore {
            raw: semaphore,
            id,
            device_id: self.id,
        }
    }

    pub fn create_fence(&self, id: FenceId, signaled: bool) -> Fence<B> {
        use gfx_hal::device::Device;
        let fence = self
            .raw
            .create_fence(signaled)
            .expect("Failed to create fence");
        Fence {
            raw: fence,
            id,
            device_id: self.id,
        }
    }

    pub fn create_image(
        &self,
        id: ImageId,
        kind: gfx_hal::image::Kind,
        mip_levels: gfx_hal::image::Level,
        format: gfx_hal::format::Format,
        tiling: gfx_hal::image::Tiling,
        usage: gfx_hal::image::Usage,
        view_caps: gfx_hal::image::ViewCapabilities,
    ) -> Image<B> {
        use gfx_hal::device::Device;
        let image = unsafe {
            self.raw
                .create_image(kind, mip_levels, format, tiling, usage, view_caps)
        }
        .expect("Failed to create image");

        Image {
            raw: image,
            id,
            device_id: self.id,
        }
    }

    pub fn allocate_memory(&self, id: MemoryId, memory_type: MemoryTypeId, size: u64) -> Memory<B> {
        use gfx_hal::device::Device;
        let memory = unsafe { self.raw.allocate_memory(memory_type, size) }.unwrap();
        Memory {
            raw: memory,
            id,
            device_id: self.id,
            size,
        }
    }

    pub fn create_image_view(
        &self,
        id: ImageViewId,
        image: &Image<B>,
        view_kind: gfx_hal::image::ViewKind,
        format: gfx_hal::format::Format,
        swizzle: gfx_hal::format::Swizzle,
        range: gfx_hal::image::SubresourceRange,
    ) -> ImageView<B> {
        use gfx_hal::device::Device;
        let image_view = unsafe {
            self.raw
                .create_image_view(&image.raw, view_kind, format, swizzle, range)
        }
        .expect("Failed to create ImageView");
        ImageView {
            raw: image_view,
            id,
            device_id: self.id,
        }
    }

    pub fn create_render_pass<'a, IA, IS, ID>(
        &self,
        id: RenderPassId,
        attachments: IA,
        subpasses: IS,
        dependencies: ID,
    ) -> RenderPass<B>
    where
        IA: IntoIterator,
        IA::Item: Borrow<pass::Attachment>,
        IA::IntoIter: ExactSizeIterator,
        IS: IntoIterator,
        IS::Item: Borrow<pass::SubpassDesc<'a>>,
        IS::IntoIter: ExactSizeIterator,
        ID: IntoIterator,
        ID::Item: Borrow<pass::SubpassDependency>,
        ID::IntoIter: ExactSizeIterator,
    {
        use gfx_hal::device::Device;
        let render_pass = unsafe {
            self.raw
                .create_render_pass(attachments, subpasses, dependencies)
        }
        .expect("Failed to create RenderPass");
        RenderPass {
            raw: render_pass,
            id,
            device_id: self.id,
        }
    }

    pub fn map_memory(
        &self,
        id: MemoryMapId,
        memory: &mut Memory<B>,
        segment: Segment,
    ) -> MemoryMap {
        use gfx_hal::device::Device;
        let memory_map =
            unsafe { self.raw.map_memory(&mut memory.raw, segment) }.expect("Failed to map memory");
        MemoryMap {
            id,
            raw: memory_map,
            memory_id: self.id,
            size: memory.size,
        }
    }

    pub fn create_framebuffer<I>(
        &self,
        id: FramebufferId,
        render_pass: &RenderPass<B>,
        attachments: I,
        extent: Extent,
    ) -> Framebuffer<B>
    where
        I: IntoIterator,
        I::Item: Borrow<B::ImageView>,
    {
        use gfx_hal::device::Device;
        let framebuffer = unsafe {
            self.raw
                .create_framebuffer(&render_pass.raw, attachments, extent)
        }
        .expect("Failed to create Framebuffer");
        Framebuffer {
            raw: framebuffer,
            render_pass_id: render_pass.id,
            id,
        }
    }

    pub fn wait_for_fence(&self, fence: &Fence<B>, timeout_ns: u64) -> Result<bool, WaitError> {
        use gfx_hal::device::Device;
        unsafe { self.raw.wait_for_fence(&fence.raw, timeout_ns) }
    }

    pub fn reset_fence(&self, fence: &mut Fence<B>) -> Result<(), OutOfMemory> {
        use gfx_hal::device::Device;
        unsafe { self.raw.reset_fence(&mut fence.raw) }
    }

    pub fn wait_idle(&self) -> Result<(), OutOfMemory> {
        use gfx_hal::device::Device;
        self.raw.wait_idle()
    }
}

pub struct QueueGroup<B>
where
    B: Backend,
{
    pub raw: gfx_hal::queue::QueueGroup<B>,
    pub id: QueueGroupId,
    pub device_id: DeviceId,
}

impl<B> QueueGroup<B>
where
    B: Backend,
{
    pub fn family(&self) -> QueueFamilyId {
        self.raw.family
    }

    #[inline]
    pub fn submit<'a, T, Ic, S, Iw, Is>(
        &mut self,
        index: usize,
        submission: Submission<Ic, Iw, Is>,
        fence: Option<&mut Fence<B>>,
    ) where
        T: 'a + Borrow<B::CommandBuffer>,
        Ic: IntoIterator<Item = &'a T>,
        S: 'a + Borrow<B::Semaphore>,
        Iw: IntoIterator<Item = (&'a S, pso::PipelineStage)>,
        Is: IntoIterator<Item = &'a S>,
    {
        use gfx_hal::queue::CommandQueue;
        unsafe { self.raw.queues[index].submit(submission, fence.map(|x| &mut x.raw)) }
    }

    pub fn present(
        &mut self,
        index: usize,
        surface: &mut Surface<B>,
        image: <B::Surface as PresentationSurface<B>>::SwapchainImage,
        wait_semaphore: Option<&mut Semaphore<B>>,
    ) -> Result<Option<Suboptimal>, PresentError> {
        use gfx_hal::queue::CommandQueue;
        let queue = &mut self.raw.queues[index];
        unsafe { queue.present(&mut surface.raw, image, wait_semaphore.map(|x| &mut x.raw)) }
    }
}

pub struct CommandPool<B>
where
    B: Backend,
{
    pub raw: B::CommandPool,
    pub id: CommandPoolId,
    pub device_id: DeviceId,
}

impl<B> CommandPool<B>
where
    B: Backend,
{
    pub fn allocate_one(&mut self, id: CommandPoolId, level: Level) -> CommandBuffer<B> {
        use gfx_hal::pool::CommandPool;
        let command_buffer = unsafe { self.raw.allocate_one(level) };
        CommandBuffer {
            raw: command_buffer,
            id,
            command_pool_id: self.id,
        }
    }
}

pub struct CommandBuffer<B>
where
    B: Backend,
{
    pub raw: B::CommandBuffer,
    pub id: CommandBufferId,
    pub command_pool_id: CommandPoolId,
}

impl<B> CommandBuffer<B>
where
    B: Backend,
{
    pub fn begin_render_pass<T>(
        &mut self,
        render_pass: &RenderPass<B>,
        framebuffer: &Framebuffer<B>,
        render_area: pso::Rect,
        clear_values: T,
        first_subpass: SubpassContents,
    ) where
        T: IntoIterator,
        T::Item: Borrow<ClearValue>,
        T::IntoIter: ExactSizeIterator,
    {
        use gfx_hal::command::CommandBuffer;
        unsafe {
            self.raw.begin_render_pass(
                &render_pass.raw,
                &framebuffer.raw,
                render_area,
                clear_values,
                first_subpass,
            );
        }
    }

    pub fn end_render_pass(&mut self) {
        use gfx_hal::command::CommandBuffer;
        unsafe {
            self.raw.end_render_pass();
        };
    }

    pub fn begin_primary(&mut self, flags: CommandBufferFlags) {
        use gfx_hal::command::CommandBuffer;
        unsafe {
            self.raw.begin_primary(flags);
        }
    }

    pub fn finish(&mut self) {
        use gfx_hal::command::CommandBuffer;
        unsafe {
            self.raw.finish();
        }
    }
}

pub struct Semaphore<B>
where
    B: Backend,
{
    pub raw: B::Semaphore,
    pub id: SemaphoreId,
    pub device_id: DeviceId,
}

pub struct Fence<B>
where
    B: Backend,
{
    pub raw: B::Fence,
    pub id: FenceId,
    pub device_id: DeviceId,
}

pub struct Image<B>
where
    B: Backend,
{
    pub raw: B::Image,
    pub id: ImageId,
    pub device_id: DeviceId,
}

pub struct ImageView<B>
where
    B: Backend,
{
    pub raw: B::ImageView,
    pub id: ImageViewId,
    pub device_id: DeviceId,
}

pub struct Memory<B>
where
    B: Backend,
{
    pub raw: B::Memory,
    pub id: MemoryId,
    pub device_id: DeviceId,
    pub size: u64,
}

pub struct ShaderModule<B>
where
    B: Backend,
{
    pub raw: B::ShaderModule,
    pub id: ShaderModuleId,
    pub device_id: DeviceId,
}

pub struct Buffer<B>
where
    B: Backend,
{
    pub raw: B::Buffer,
    pub id: BufferId,
    pub device_id: DeviceId,
}

pub struct MemoryMap {
    pub raw: *mut u8,
    pub id: MemoryMapId,
    pub memory_id: MemoryId,
    pub size: u64,
}

pub struct RenderPass<B>
where
    B: Backend,
{
    pub raw: B::RenderPass,
    pub id: RenderPassId,
    pub device_id: DeviceId,
}

pub struct Framebuffer<B>
where
    B: Backend,
{
    pub raw: B::Framebuffer,
    pub id: FramebufferId,
    pub render_pass_id: RenderPassId,
}
