use crate::hal::AttributeDesc;
use gfx_hal::adapter::MemoryProperties;
use gfx_hal::command::{ClearValue, CommandBufferFlags, Level, SubpassContents};
use gfx_hal::device::{OutOfMemory, WaitError};
use gfx_hal::format::{ChannelType, Format};
use gfx_hal::image::Extent;
use gfx_hal::memory::{Requirements, Segment};
use gfx_hal::pool::CommandPoolCreateFlags;
use gfx_hal::pso::{
    DescriptorSetLayoutBinding, InputAssemblerDesc, Primitive, PrimitiveAssemblerDesc, Rasterizer,
    VertexBufferDesc,
};
use gfx_hal::queue::{QueueFamily, QueueFamilyId, QueuePriority, Submission};
use gfx_hal::window::{
    AcquireError, PresentError, PresentationSurface, Suboptimal, SurfaceCapabilities,
    SwapchainConfig, SwapchainError,
};
use gfx_hal::{pass, pso, Backend, Features, Limits, MemoryTypeId};
use std::borrow::Borrow;

const RENDER_PIPELINE_DESCRIPTOR_RANGE_MAX: usize = 32;
const RENDER_PIPELINE_DESCRIPTOR_SET_MAX: usize = 64;
const SHADER_MODULE_ENTRY_NAME: &str = "main";

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
pub type RenderPipelineId = u64;
pub type DescriptorSetId = u64;

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

    pub fn create_render_pipeline(
        &self,
        id: RenderPipelineId,
        desc: RenderPipelineDesc<B>,
    ) -> RenderPipeline<B> {
        use gfx_hal::device::Device;
        use gfx_hal::pso::DescriptorPool;
        let descriptor_ranges = vec![
            gfx_hal::pso::DescriptorRangeDesc {
                ty: gfx_hal::pso::DescriptorType::Buffer {
                    ty: gfx_hal::pso::BufferDescriptorType::Uniform,
                    format: gfx_hal::pso::BufferDescriptorFormat::Structured {
                        dynamic_offset: false,
                    },
                },
                count: RENDER_PIPELINE_DESCRIPTOR_RANGE_MAX,
            },
            gfx_hal::pso::DescriptorRangeDesc {
                ty: gfx_hal::pso::DescriptorType::Sampler,
                count: RENDER_PIPELINE_DESCRIPTOR_RANGE_MAX,
            },
            gfx_hal::pso::DescriptorRangeDesc {
                ty: gfx_hal::pso::DescriptorType::Image {
                    ty: gfx_hal::pso::ImageDescriptorType::Sampled { with_sampler: true },
                },
                count: RENDER_PIPELINE_DESCRIPTOR_RANGE_MAX,
            },
        ];
        let mut descriptor_pool = unsafe {
            self.raw.create_descriptor_pool(
                RENDER_PIPELINE_DESCRIPTOR_SET_MAX,
                descriptor_ranges,
                gfx_hal::pso::DescriptorPoolCreateFlags::empty(),
            )
        }
        .expect("Failed to create DescriptorPool");

        let descriptor_set_layout = unsafe {
            self.raw
                .create_descriptor_set_layout(desc.shader.bindings, &[])
        }
        .expect("Failed to create DescriptorSetLayout");

        let descriptor_set = unsafe { descriptor_pool.allocate_set(&descriptor_set_layout) }
            .expect("Failed to DescriptorSet");

        let mut descriptor_set_layouts = vec![descriptor_set_layout];
        let pipeline_layout = unsafe {
            self.raw
                .create_pipeline_layout(&descriptor_set_layouts, &[])
        }
        .expect("Failed to create PipelineLayout");

        let subpass = gfx_hal::pass::Subpass {
            index: 0,
            main_pass: &desc.main_pass.raw,
        };

        let buffers = desc.shader.create_vertex_buffers();
        let attributes = desc.shader.create_attribute();
        let mut pipeline_desc = gfx_hal::pso::GraphicsPipelineDesc::new(
            PrimitiveAssemblerDesc::Vertex {
                buffers: &buffers,
                attributes: &attributes,
                input_assembler: InputAssemblerDesc {
                    primitive: desc.primitive,
                    with_adjacency: false,
                    restart_index: None,
                },
                vertex: gfx_hal::pso::EntryPoint {
                    entry: SHADER_MODULE_ENTRY_NAME,
                    module: &desc.shader.vertex_module.raw,
                    specialization: gfx_hal::pso::Specialization::default(),
                },
                geometry: None,
                tessellation: None,
            },
            desc.rasterization,
            Some(gfx_hal::pso::EntryPoint {
                entry: SHADER_MODULE_ENTRY_NAME,
                module: &desc.shader.fragment_module.raw,
                specialization: gfx_hal::pso::Specialization::default(),
            }),
            &pipeline_layout,
            subpass,
        );

        // TODO: Set by desc
        pipeline_desc
            .blender
            .targets
            .push(gfx_hal::pso::ColorBlendDesc {
                mask: gfx_hal::pso::ColorMask::ALL,
                blend: Some(gfx_hal::pso::BlendState::ALPHA),
            });
        pipeline_desc.depth_stencil.depth = Some(gfx_hal::pso::DepthTest {
            fun: gfx_hal::pso::Comparison::LessEqual,
            write: true,
        });

        let pipeline = unsafe { self.raw.create_graphics_pipeline(&pipeline_desc, None) }
            .expect("Failed to create GraphicsPipeline");
        RenderPipeline {
            id,
            device_id: self.id,
            raw_pipeline: pipeline,
            raw_descriptor_pool: descriptor_pool,
            raw_descriptor_set_layout: descriptor_set_layouts.remove(0),
            raw_descriptor_set: descriptor_set,
        }
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

pub struct RenderPipelineDesc<'a, B>
where
    B: Backend,
{
    pub shader: ShaderDesc<'a, B>,
    pub main_pass: &'a RenderPass<B>,
    pub rasterization: Rasterizer,
    pub primitive: Primitive,
}

pub struct RenderPipeline<B>
where
    B: Backend,
{
    pub raw_pipeline: B::GraphicsPipeline,
    pub raw_descriptor_pool: B::DescriptorPool,
    pub raw_descriptor_set: B::DescriptorSet,
    pub raw_descriptor_set_layout: B::DescriptorSetLayout,
    pub id: RenderPipelineId,
    pub device_id: DeviceId,
}

pub struct ShaderDesc<'a, B>
where
    B: Backend,
{
    pub vertex_module: &'a ShaderModule<B>,
    pub fragment_module: &'a ShaderModule<B>,
    pub attributes: &'a Vec<AttributeDesc>,
    pub bindings: &'a Vec<DescriptorSetLayoutBinding>,
}

impl<'a, B> ShaderDesc<'a, B>
where
    B: Backend,
{
    pub fn create_vertex_buffers(&self) -> Vec<gfx_hal::pso::VertexBufferDesc> {
        self.attributes
            .iter()
            .enumerate()
            .map(|(i, attr)| VertexBufferDesc {
                binding: i as u32,
                stride: attr.stride,
                rate: gfx_hal::pso::VertexInputRate::Vertex,
            })
            .collect()
    }

    pub fn create_attribute(&self) -> Vec<gfx_hal::pso::AttributeDesc> {
        self.attributes.iter().map(|x| x.desc).collect::<Vec<_>>()
    }
}

pub struct DescriptorSet<B>
where
    B: Backend,
{
    pub raw: B::DescriptorSet,
    pub id: DescriptorSetId,
    pub device_id: DeviceId,
}
