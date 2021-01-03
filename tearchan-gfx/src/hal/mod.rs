use crate::hal::global::{
    AdapterId, BufferId, CommandBufferId, CommandPoolId, DeviceId, FenceId, ImageId, ImageViewId,
    MemoryId, MemoryMapId, QueueGroupId, RenderPassId, SemaphoreId, ShaderModuleId,
};
use crate::registry::Registry;
use crate::utils::find_memory_type;
use gfx_hal::command::Level;
use gfx_hal::format::{Format, Swizzle};
use gfx_hal::image::{Kind, SubresourceRange, Tiling, Usage, ViewCapabilities, ViewKind};
use gfx_hal::memory::{Properties, Requirements, Segment};
use gfx_hal::pool::CommandPoolCreateFlags;
use gfx_hal::queue::{QueueFamilyId, QueuePriority};
use gfx_hal::window::{
    AcquireError, PresentationSurface, Suboptimal, SurfaceCapabilities, SwapchainConfig,
    SwapchainError,
};
use gfx_hal::{pass, Backend, Features, MemoryTypeId};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use winit::window::Window;

pub mod backend;
pub mod queue;
pub mod surface;
pub mod swapchain;

struct Global<B>
where
    B: Backend,
{
    instance: global::Instance<B>,
    adapters: Registry<global::Adapter<B>>,
    surface: Option<RefCell<global::Surface<B>>>,
    devices: Registry<global::Device<B>>,
    queue_groups: Registry<global::QueueGroup<B>>,
    command_pools: Registry<global::CommandPool<B>>,
    command_buffers: Registry<global::CommandBuffer<B>>,
    semaphores: Registry<global::Semaphore<B>>,
    fences: Registry<global::Fence<B>>,
    images: Registry<global::Image<B>>,
    image_views: Registry<global::ImageView<B>>,
    memories: Registry<global::Memory<B>>,
    shader_modules: Registry<global::ShaderModule<B>>,
    buffers: Registry<global::Buffer<B>>,
    memory_maps: Registry<global::MemoryMap>,
    render_passes: Registry<global::RenderPass<B>>,
}

mod global;

pub struct InstanceCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    adapters: Vec<AdapterCommon<B>>,
    surface: SurfaceCommon<B>,
}

impl<B> InstanceCommon<B>
where
    B: Backend,
{
    pub fn new(window: &Window) -> InstanceCommon<B> {
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .body()
                .unwrap()
                .append_child(&WindowExtWebSys::canvas(window))
                .unwrap();
        }

        let (instance, surface, adapters) = {
            use gfx_hal::Instance;
            let instance =
                B::Instance::create("tearchan", 1).expect("Failed to create an instance!");
            let surface = unsafe {
                instance
                    .create_surface(window)
                    .expect("Failed to create a surface!")
            };
            let adapters = instance.enumerate_adapters();
            (instance, surface, adapters)
        };

        let global = Arc::new(Mutex::new(Global {
            instance: global::Instance { raw: instance },
            adapters: Registry::default(),
            surface: None,
            devices: Registry::default(),
            queue_groups: Registry::default(),
            command_pools: Registry::default(),
            command_buffers: Registry::default(),
            semaphores: Registry::default(),
            fences: Registry::default(),
            images: Registry::default(),
            image_views: Registry::default(),
            memories: Registry::default(),
            shader_modules: Registry::default(),
            buffers: Registry::default(),
            memory_maps: Registry::default(),
            render_passes: Registry::default(),
        }));

        let adapters = {
            let mut write = global.try_lock().unwrap();
            write.surface = Some(RefCell::new(global::Surface { raw: surface }));
            adapters
                .into_iter()
                .map(|adapter| {
                    let id = write.adapters.gen_id();
                    write
                        .adapters
                        .register(id, global::Adapter { raw: adapter, id });
                    AdapterCommon {
                        global: Arc::clone(&global),
                        id,
                    }
                })
                .collect()
        };
        let surface = SurfaceCommon {
            global: Arc::clone(&global),
        };

        InstanceCommon {
            global,
            adapters,
            surface,
        }
    }

    pub fn adapters(&self) -> &Vec<AdapterCommon<B>> {
        &self.adapters
    }

    pub fn surface(&self) -> &SurfaceCommon<B> {
        &self.surface
    }
}

pub struct AdapterCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: AdapterId,
}

impl<B> AdapterCommon<B>
where
    B: Backend,
{
    pub fn find_queue_family(&self) -> QueueFamilyId {
        let global = self.global.try_lock().unwrap();
        let adapter = global.adapters.read(self.id);
        let surface = global.surface.as_ref().unwrap().borrow();
        adapter.find_queue_family(&surface)
    }

    pub fn create_device(
        &self,
        families: &[(QueueFamilyId, &[QueuePriority])],
        requested_features: Features,
    ) -> (DeviceCommon<B>, Vec<QueueGroupCommon<B>>) {
        let global = self.global.try_lock().unwrap();
        let adapter = global.adapters.read(self.id);
        let device_id = global.devices.gen_id();
        let (device, queue_groups) = adapter.create_device(device_id, families, requested_features);
        global.devices.register(device_id, device);
        (
            DeviceCommon {
                global: Arc::clone(&self.global),
                id: device_id,
            },
            queue_groups
                .into_iter()
                .map(|queue_group| {
                    let id = global.queue_groups.gen_id();
                    global.queue_groups.register(
                        id,
                        global::QueueGroup {
                            raw: queue_group,
                            id,
                            device_id,
                        },
                    );
                    QueueGroupCommon {
                        global: Arc::clone(&self.global),
                        id,
                    }
                })
                .collect(),
        )
    }
}

pub struct SurfaceCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
}

impl<B> SurfaceCommon<B>
where
    B: Backend,
{
    pub fn capabilities(&self, adapter: &AdapterCommon<B>) -> SurfaceCapabilities {
        let global = self.global.try_lock().unwrap();
        let adapter = global.adapters.read(adapter.id);
        let surface = global.surface.as_ref().unwrap().borrow();
        surface.capabilities(&adapter)
    }

    pub fn find_support_format(&self, adapter: &AdapterCommon<B>) -> Format {
        let global = self.global.try_lock().unwrap();
        let adapter = global.adapters.read(adapter.id);
        let surface = global.surface.as_ref().unwrap().borrow();
        surface.find_support_format(&adapter)
    }

    pub fn configure_swapchain(
        &self,
        device: &DeviceCommon<B>,
        swap_config: SwapchainConfig,
    ) -> Result<(), SwapchainError> {
        let global = self.global.try_lock().unwrap();
        let mut surface = global.surface.as_ref().unwrap().borrow_mut();
        let device = global.devices.read(device.id);
        surface.configure_swapchain(&device, swap_config)
    }

    pub fn acquire_image(
        &self,
        timeout_ns: u64,
    ) -> Result<
        (
            <<B as Backend>::Surface as PresentationSurface<B>>::SwapchainImage,
            Option<Suboptimal>,
        ),
        AcquireError,
    > {
        let global = self.global.try_lock().unwrap();
        let mut surface = global.surface.as_ref().unwrap().borrow_mut();
        surface.acquire_image(timeout_ns)
    }
}

pub struct DeviceCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: DeviceId,
}

impl<B> DeviceCommon<B>
where
    B: Backend,
{
    pub fn create_command_pool(
        &self,
        family: QueueFamilyId,
        create_flags: CommandPoolCreateFlags,
    ) -> CommandPoolCommon<B> {
        let global = self.global.try_lock().unwrap();
        let device = global.devices.read(self.id);
        let id = global.command_pools.gen_id();
        global
            .command_pools
            .register(id, device.create_command_pool(id, family, create_flags));
        CommandPoolCommon {
            global: Arc::clone(&self.global),
            id,
        }
    }

    pub fn create_semaphore(&self) -> SemaphoreCommon<B> {
        let global = self.global.try_lock().unwrap();
        let device = global.devices.read(self.id);
        let id = global.semaphores.gen_id();
        global.semaphores.register(id, device.create_semaphore(id));
        SemaphoreCommon {
            global: Arc::clone(&self.global),
            id,
        }
    }

    pub fn create_fence(&self, signaled: bool) -> FenceCommon<B> {
        let global = self.global.try_lock().unwrap();
        let device = global.devices.read(self.id);
        let id = global.fences.gen_id();
        global
            .fences
            .register(id, device.create_fence(id, signaled));
        FenceCommon {
            global: Arc::clone(&self.global),
            id,
        }
    }

    pub fn create_image(
        &self,
        kind: gfx_hal::image::Kind,
        mip_levels: gfx_hal::image::Level,
        format: gfx_hal::format::Format,
        tiling: gfx_hal::image::Tiling,
        usage: gfx_hal::image::Usage,
        view_caps: gfx_hal::image::ViewCapabilities,
    ) -> ImageCommon<B> {
        let global = self.global.try_lock().unwrap();
        let device = global.devices.read(self.id);
        let id = global.images.gen_id();
        global.images.register(
            id,
            device.create_image(id, kind, mip_levels, format, tiling, usage, view_caps),
        );
        ImageCommon {
            global: Arc::clone(&self.global),
            id,
        }
    }

    pub fn allocate_memory(&self, memory_type: MemoryTypeId, size: u64) -> MemoryCommon<B> {
        let global = self.global.try_lock().unwrap();
        let device = global.devices.read(self.id);
        let id = global.memories.gen_id();
        global
            .memories
            .register(id, device.allocate_memory(id, memory_type, size));
        MemoryCommon {
            global: Arc::clone(&self.global),
            id,
        }
    }

    pub fn create_image_view(
        &self,
        image: &ImageCommon<B>,
        view_kind: gfx_hal::image::ViewKind,
        format: gfx_hal::format::Format,
        swizzle: gfx_hal::format::Swizzle,
        range: gfx_hal::image::SubresourceRange,
    ) -> ImageViewCommon<B> {
        let global = self.global.try_lock().unwrap();
        let device = global.devices.read(self.id);
        let image = global.images.read(image.id);
        let id = global.image_views.gen_id();
        global.image_views.register(
            id,
            device.create_image_view(id, &image, view_kind, format, swizzle, range),
        );
        ImageViewCommon {
            global: Arc::clone(&self.global),
            id,
        }
    }

    pub fn create_texture(
        &self,
        format: Format,
        usage: Usage,
        color_range: SubresourceRange,
        width: u32,
        height: u32,
    ) -> TextureCommon<B> {
        let global = self.global.try_lock().unwrap();
        let device = global.devices.read(self.id);

        let kind = Kind::D2(width, height, 1, 1);
        let image_id = global.images.gen_id();
        let mut image = device.create_image(
            image_id,
            kind,
            1,
            format,
            Tiling::Optimal,
            usage,
            ViewCapabilities::empty(),
        );

        let image_req = device.get_image_requirements(&image);
        let device_type = find_memory_type(
            &device.memory_properties.memory_types,
            &image_req,
            Properties::DEVICE_LOCAL,
        );
        let image_memory_id = global.memories.gen_id();
        let image_memory = device.allocate_memory(image_memory_id, device_type, image_req.size);

        device.bind_image_memory(&image_memory, 0, &mut image);

        let image_view_id = global.image_views.gen_id();
        let image_view = device.create_image_view(
            image_view_id,
            &image,
            ViewKind::D2,
            format,
            Swizzle::NO,
            color_range.clone(),
        );

        global.images.register(image_id, image);
        global.memories.register(image_memory_id, image_memory);
        global.image_views.register(image_view_id, image_view);
        TextureCommon {
            image: ImageCommon {
                global: Arc::clone(&self.global),
                id: image_id,
            },
            image_view: ImageViewCommon {
                global: Arc::clone(&self.global),
                id: image_view_id,
            },
            image_memory: MemoryCommon {
                global: Arc::clone(&self.global),
                id: image_memory_id,
            },
            color_range,
            format,
            width,
            height,
        }
    }

    pub fn create_render_pass<'a, IA, IS, ID>(
        &self,
        attachments: IA,
        subpasses: IS,
        dependencies: ID,
    ) -> RenderPassCommon<B>
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
        let global = self.global.try_lock().unwrap();
        let device = global.devices.read(self.id);
        let id = global.render_passes.gen_id();
        global.render_passes.register(
            id,
            device.create_render_pass(id, attachments, subpasses, dependencies),
        );
        RenderPassCommon {
            global: Arc::clone(&self.global),
            id,
        }
    }
}

pub struct QueueGroupCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: QueueGroupId,
}

impl<B> QueueGroupCommon<B>
where
    B: Backend,
{
    pub fn family(&self) -> QueueFamilyId {
        let global = self.global.try_lock().unwrap();
        let queue_group = global.queue_groups.read(self.id);
        queue_group.family()
    }
}

pub struct CommandPoolCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: CommandPoolId,
}

impl<B> CommandPoolCommon<B>
where
    B: Backend,
{
    pub fn allocate_one(&self, level: Level) -> CommandBufferCommon<B> {
        let global = self.global.try_lock().unwrap();
        let mut command_pool = global.command_pools.write(self.id);
        let id = global.command_buffers.gen_id();
        global
            .command_buffers
            .register(id, command_pool.allocate_one(id, level));
        CommandBufferCommon {
            global: Arc::clone(&self.global),
            id,
        }
    }
}

impl<B> Drop for CommandPoolCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::device::Device;

            let global = self.global.try_lock().unwrap();
            let command_pool = global.command_pools.unregister(self.id).unwrap();
            let device = global.devices.write(command_pool.device_id);
            unsafe {
                device.raw.destroy_command_pool(command_pool.raw);
            }
        }
    }
}

pub struct CommandBufferCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: CommandBufferId,
}

impl<B> Drop for CommandBufferCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::pool::CommandPool;

            let global = self.global.try_lock().unwrap();
            let command_buffer = global.command_buffers.unregister(self.id).unwrap();
            let mut command_pool = global.command_pools.write(command_buffer.command_pool_id);
            unsafe {
                command_pool.raw.free(vec![command_buffer.raw]);
            }
        }
    }
}

pub struct SemaphoreCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: SemaphoreId,
}

impl<B> Drop for SemaphoreCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::device::Device;

            let global = self.global.try_lock().unwrap();
            let semaphore = global.semaphores.unregister(self.id).unwrap();
            let device = global.devices.write(semaphore.device_id);
            unsafe {
                device.raw.destroy_semaphore(semaphore.raw);
            }
        }
    }
}

pub struct FenceCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: FenceId,
}

impl<B> Drop for FenceCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::device::Device;

            let global = self.global.try_lock().unwrap();
            let fence = global.fences.unregister(self.id).unwrap();
            let device = global.devices.write(fence.device_id);
            unsafe {
                device.raw.destroy_fence(fence.raw);
            }
        }
    }
}

pub struct ImageCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: ImageId,
}

impl<B> ImageCommon<B>
where
    B: Backend,
{
    pub fn get_requirements(&self) -> Requirements {
        let global = self.global.try_lock().unwrap();
        let image = global.images.read(self.id);
        let device = global.devices.read(image.device_id);
        device.get_image_requirements(&image)
    }
}

impl<B> Drop for ImageCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::device::Device;

            let global = self.global.try_lock().unwrap();
            let image = global.images.unregister(self.id).unwrap();
            let device = global.devices.write(image.device_id);
            unsafe {
                device.raw.destroy_image(image.raw);
            }
        }
    }
}

pub struct ImageViewCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: ImageViewId,
}

impl<B> Drop for ImageViewCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::device::Device;

            let global = self.global.try_lock().unwrap();
            let image_view = global.image_views.unregister(self.id).unwrap();
            let device = global.devices.write(image_view.device_id);
            unsafe {
                device.raw.destroy_image_view(image_view.raw);
            }
        }
    }
}

pub struct MemoryCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: MemoryId,
}

impl<B> MemoryCommon<B>
where
    B: Backend,
{
    pub fn bind_image_memory(&self, offset: u64, image: &ImageCommon<B>) {
        let global = self.global.try_lock().unwrap();
        let memory = global.memories.read(self.id);
        let device = global.devices.read(memory.device_id);
        let mut image = global.images.write(image.id);
        device.bind_image_memory(&memory, offset, &mut image)
    }

    pub fn map_memory(&self, segment: Segment) -> MemoryMapCommon<B> {
        let global = self.global.try_lock().unwrap();
        let mut memory = global.memories.write(self.id);
        let device = global.devices.read(memory.device_id);
        let id = global.memory_maps.gen_id();
        global
            .memory_maps
            .register(id, device.map_memory(id, &mut memory, segment));
        MemoryMapCommon {
            global: Arc::clone(&self.global),
            id,
        }
    }
}

impl<B> Drop for MemoryCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::device::Device;
            let global = self.global.try_lock().unwrap();
            let memory = global.memories.unregister(self.id).unwrap();
            let device = global.devices.write(memory.device_id);
            unsafe {
                device.raw.free_memory(memory.raw);
            }
        }
    }
}

pub struct ShaderModuleCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: ShaderModuleId,
}

impl<B> Drop for ShaderModuleCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::device::Device;
            let global = self.global.try_lock().unwrap();
            let shader_module = global.shader_modules.unregister(self.id).unwrap();
            let device = global.devices.write(shader_module.device_id);
            unsafe {
                device.raw.destroy_shader_module(shader_module.raw);
            }
        }
    }
}

#[derive(Debug)]
pub enum TextureError {
    InvalidCopyRange,
}

pub struct TextureCommon<B>
where
    B: Backend,
{
    image: ImageCommon<B>,
    image_view: ImageViewCommon<B>,
    image_memory: MemoryCommon<B>,
    color_range: SubresourceRange,
    format: Format,
    width: u32,
    height: u32,
}

impl<B> TextureCommon<B>
where
    B: Backend,
{
    pub fn image(&self) -> &ImageCommon<B> {
        &self.image
    }

    pub fn image_view(&self) -> &ImageViewCommon<B> {
        &self.image_view
    }

    pub fn format(&self) -> &Format {
        &self.format
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

pub struct BufferCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    device_id: DeviceId,
    id: BufferId,
    size: u64,
}

impl<B> Drop for BufferCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::device::Device;
            let global = self.global.try_lock().unwrap();
            let device = global.devices.write(self.device_id);
            let buffer = global.buffers.unregister(self.id).unwrap();
            unsafe {
                device.raw.destroy_buffer(buffer.raw);
            }
        }
    }
}

pub struct MemoryMapCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: MemoryMapId,
}

impl<B> Drop for MemoryMapCommon<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        if !std::thread::panicking() {
            use gfx_hal::device::Device;
            let global = self.global.try_lock().unwrap();
            let memory_map = global.memory_maps.unregister(self.id).unwrap();
            let mut memory = global.memories.write(memory_map.memory_id);
            let device = global.devices.write(memory.device_id);
            unsafe {
                device.raw.unmap_memory(&mut memory.raw);
            }
        }
    }
}

pub struct RenderPassCommon<B>
where
    B: Backend,
{
    global: Arc<Mutex<Global<B>>>,
    id: RenderPassId,
}
