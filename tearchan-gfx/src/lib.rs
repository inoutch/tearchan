use crate::hal::backend::Backend;
use crate::hal::swapchain::{SwapchainCommon, SwapchainFrameCommon};
use crate::hal::{
    AdapterCommon, CommandBufferCommon, CommandPoolCommon, CommandQueueCommon, DeviceCommon,
    FenceCommon, ImageCommon, ImageViewCommon, InstanceCommon, MemoryCommon, MemoryMapCommon,
    QueueGroupCommon, RenderPassCommon, RenderPipelineCommon, RenderPipelineDescCommon,
    SemaphoreCommon, ShaderDescCommon, ShaderModuleCommon, SurfaceCommon, TextureCommon,
    UniformBufferCommon,
};

pub mod bitmap;
pub mod context;
pub mod hal;
pub mod registry;
pub mod setup;
pub mod shader_program;
pub mod types;
pub mod utils;

pub type Instance = InstanceCommon<Backend>;
pub type Adapter = AdapterCommon<Backend>;
pub type Surface = SurfaceCommon<Backend>;
pub type Device = DeviceCommon<Backend>;
pub type QueueGroup = QueueGroupCommon<Backend>;
pub type CommandQueue = CommandQueueCommon<Backend>;
pub type CommandPool = CommandPoolCommon<Backend>;
pub type CommandBuffer = CommandBufferCommon<Backend>;
pub type Semaphore = SemaphoreCommon<Backend>;
pub type Fence = FenceCommon<Backend>;
pub type Image = ImageCommon<Backend>;
pub type ImageView = ImageViewCommon<Backend>;
pub type Memory = MemoryCommon<Backend>;
pub type ShaderModule = ShaderModuleCommon<Backend>;
pub type Texture = TextureCommon<Backend>;
pub type MemoryMap = MemoryMapCommon<Backend>;
pub type Swapchain = SwapchainCommon<Backend>;
pub type SwapchainFrame = SwapchainFrameCommon<Backend>;
pub type RenderPass = RenderPassCommon<Backend>;
pub type RenderPipeline = RenderPipelineCommon<Backend>;
pub type RenderPipelineDesc<'a> = RenderPipelineDescCommon<'a, Backend>;
pub type ShaderDesc = ShaderDescCommon<Backend>;
pub type UniformBuffer<T> = UniformBufferCommon<Backend, T>;
