use crate::core::graphic::hal::renderer_api::RendererApiCommon;
use gfx_hal::adapter::{Adapter, MemoryType, PhysicalDevice};
use gfx_hal::command::CommandBuffer;
use gfx_hal::device::Device;
use gfx_hal::format::{ChannelType, Swizzle};
use gfx_hal::pool::CommandPool;
use gfx_hal::pso::{Rect, Viewport};
use gfx_hal::queue::{CommandQueue, QueueFamily, QueueGroup, Submission};
use gfx_hal::window::{PresentationSurface, Surface};
use gfx_hal::{window, Backend, Instance, Limits};
use nalgebra_glm::{vec4, Vec4};
use std::borrow::Borrow;
use std::iter;
use std::mem::ManuallyDrop;
use std::rc::Rc;

pub struct Renderer<B: gfx_hal::Backend> {
    instance: Option<B::Instance>,
    surface: ManuallyDrop<B::Surface>,
    adapter: Adapter<B>,
    surface_format: gfx_hal::format::Format,
    dimensions: window::Extent2D,
    viewport: gfx_hal::pso::Viewport,
    cmd_pools: Vec<B::CommandPool>,
    cmd_buffers: Vec<B::CommandBuffer>,
    submission_complete_semaphores: Vec<B::Semaphore>,
    submission_complete_fences: Vec<B::Fence>,
    frames_in_flight: usize,
    frame: u64,
    context: RendererApiContext<B>,
    static_context: RendererApiStaticContext,
    depth_image: ManuallyDrop<B::Image>,
    depth_memory: ManuallyDrop<B::Memory>,
    depth_image_view: ManuallyDrop<B::ImageView>,
}

impl<B> Renderer<B>
where
    B: gfx_hal::Backend,
{
    pub fn new(
        instance: Option<B::Instance>,
        adapter: Adapter<B>,
        mut surface: B::Surface,
        default_dimensions: window::Extent2D,
    ) -> Renderer<B> {
        let memory_types = adapter.physical_device.memory_properties().memory_types;
        let limits = adapter.physical_device.limits();
        let static_context = RendererApiStaticContext {
            memory_types,
            limits,
        };

        // Build a new device and associated command queues
        let family = adapter
            .queue_families
            .iter()
            .find(|family| {
                surface.supports_queue_family(family) && family.queue_type().supports_graphics()
            })
            .unwrap();
        let mut gpu = unsafe {
            adapter
                .physical_device
                .open(&[(family, &[1.0])], gfx_hal::Features::empty())
                .unwrap()
        };
        let queue_group = gpu.queue_groups.pop().unwrap();
        let device = gpu.device;

        let command_pool = unsafe {
            device.create_command_pool(
                queue_group.family,
                gfx_hal::pool::CommandPoolCreateFlags::empty(),
            )
        }
        .expect("Can't create command pool");

        let caps = surface.capabilities(&adapter.physical_device);
        let formats = surface.supported_formats(&adapter.physical_device);
        let surface_format = formats.map_or(gfx_hal::format::Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .copied()
                .unwrap_or(formats[0])
        });

        let swap_config =
            window::SwapchainConfig::from_caps(&caps, surface_format, default_dimensions);
        let extent = swap_config.extent;
        unsafe {
            surface
                .configure_swapchain(&device, swap_config)
                .expect("Can't configure swapchain");
        };

        let (depth_image, depth_memory, depth_image_view, depth_stencil_format) =
            create_depth::<B>(&device, extent, &static_context.memory_types);

        let first_render_pass = {
            let attachment = gfx_hal::pass::Attachment {
                format: Some(surface_format),
                samples: 1,
                ops: gfx_hal::pass::AttachmentOps::new(
                    gfx_hal::pass::AttachmentLoadOp::Clear,
                    gfx_hal::pass::AttachmentStoreOp::Store,
                ),
                stencil_ops: gfx_hal::pass::AttachmentOps::DONT_CARE,
                layouts: gfx_hal::image::Layout::Undefined..gfx_hal::image::Layout::Present,
            };

            let depth_attachment = gfx_hal::pass::Attachment {
                format: Some(depth_stencil_format),
                samples: 1,
                ops: gfx_hal::pass::AttachmentOps::new(
                    gfx_hal::pass::AttachmentLoadOp::Clear,
                    gfx_hal::pass::AttachmentStoreOp::Store,
                ),
                stencil_ops: gfx_hal::pass::AttachmentOps::DONT_CARE,
                layouts: gfx_hal::image::Layout::Undefined
                    ..gfx_hal::image::Layout::DepthStencilAttachmentOptimal,
            };

            let subpass = gfx_hal::pass::SubpassDesc {
                colors: &[(0, gfx_hal::image::Layout::ColorAttachmentOptimal)],
                depth_stencil: Some(&(1, gfx_hal::image::Layout::DepthStencilAttachmentOptimal)),
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            ManuallyDrop::new(
                unsafe {
                    device.create_render_pass(&[attachment, depth_attachment], &[subpass], &[])
                }
                .expect("Can't create render pass"),
            )
        };

        let render_pass = {
            let attachment = gfx_hal::pass::Attachment {
                format: Some(surface_format),
                samples: 1,
                ops: gfx_hal::pass::AttachmentOps::new(
                    gfx_hal::pass::AttachmentLoadOp::Load,
                    gfx_hal::pass::AttachmentStoreOp::Store,
                ),
                stencil_ops: gfx_hal::pass::AttachmentOps::DONT_CARE,
                layouts: gfx_hal::image::Layout::Undefined..gfx_hal::image::Layout::Present,
            };

            let depth_attachment = gfx_hal::pass::Attachment {
                format: Some(depth_stencil_format),
                samples: 1,
                ops: gfx_hal::pass::AttachmentOps::new(
                    gfx_hal::pass::AttachmentLoadOp::Load,
                    gfx_hal::pass::AttachmentStoreOp::Store,
                ),
                stencil_ops: gfx_hal::pass::AttachmentOps::DONT_CARE,
                layouts: gfx_hal::image::Layout::Undefined
                    ..gfx_hal::image::Layout::DepthStencilAttachmentOptimal,
            };

            let subpass = gfx_hal::pass::SubpassDesc {
                colors: &[(0, gfx_hal::image::Layout::ColorAttachmentOptimal)],
                depth_stencil: Some(&(1, gfx_hal::image::Layout::DepthStencilAttachmentOptimal)),
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            ManuallyDrop::new(
                unsafe {
                    device.create_render_pass(&[attachment, depth_attachment], &[subpass], &[])
                }
                .expect("Can't create render pass"),
            )
        };

        let frames_in_flight = 3;

        let mut submission_complete_semaphores = Vec::with_capacity(frames_in_flight);
        let mut submission_complete_fences = Vec::with_capacity(frames_in_flight);
        let mut cmd_pools = Vec::with_capacity(frames_in_flight);
        let mut cmd_buffers = Vec::with_capacity(frames_in_flight);

        cmd_pools.push(command_pool);
        for _ in 1..frames_in_flight {
            unsafe {
                cmd_pools.push(
                    device
                        .create_command_pool(
                            queue_group.family,
                            gfx_hal::pool::CommandPoolCreateFlags::empty(),
                        )
                        .expect("Can't create command pool"),
                );
            }
        }

        cmd_pools[0..frames_in_flight]
            .iter_mut()
            .for_each(|cmd_pool| {
                submission_complete_semaphores.push(
                    device
                        .create_semaphore()
                        .expect("Could not create semaphore"),
                );
                submission_complete_fences
                    .push(device.create_fence(true).expect("Could not create fence"));
                cmd_buffers
                    .push(unsafe { cmd_pool.allocate_one(gfx_hal::command::Level::Primary) });
            });

        // Rendering setup
        let viewport = gfx_hal::pso::Viewport {
            rect: gfx_hal::pso::Rect {
                x: 0,
                y: 0,
                w: extent.width as _,
                h: extent.height as _,
            },
            depth: 0.0..1.0,
        };

        Renderer {
            instance,
            surface: ManuallyDrop::new(surface),
            adapter,
            context: RendererApiContext {
                device: Rc::new(device),
                queue_group,
                render_pass,
                first_render_pass,
                use_first_render_pass: true,
                clear_color: vec4(0.0f32, 0.0f32, 0.0f32, 1.0f32),
                clear_depth: 0.0f32,
            },
            surface_format,
            dimensions: default_dimensions,
            viewport,
            frames_in_flight,
            submission_complete_semaphores,
            submission_complete_fences,
            cmd_pools,
            cmd_buffers,
            frame: 0,
            static_context,
            depth_image: ManuallyDrop::new(depth_image),
            depth_memory: ManuallyDrop::new(depth_memory),
            depth_image_view: ManuallyDrop::new(depth_image_view),
        }
    }

    pub fn recreate_swapchain(&mut self) {
        let caps = self.surface.capabilities(&self.adapter.physical_device);
        let swap_config =
            window::SwapchainConfig::from_caps(&caps, self.surface_format, self.dimensions);
        println!("{:?}", swap_config);
        let extent = swap_config.extent.to_extent();

        unsafe {
            self.surface
                .configure_swapchain(&self.context.device, swap_config)
                .expect("Can't create swapchain");
        }

        self.viewport.rect.w = extent.width as _;
        self.viewport.rect.h = extent.height as _;
    }

    pub fn render<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut RendererApiCommon<B>) -> (),
    {
        self.context.use_first_render_pass = true;

        let frame_idx = self.frame as usize % self.frames_in_flight;
        let surface_image = unsafe {
            match self.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.recreate_swapchain();
                    return;
                }
            }
        };

        let attachments = vec![surface_image.borrow(), &self.depth_image_view];
        let framebuffer = unsafe {
            self.context
                .device
                .create_framebuffer(
                    &self.context.render_pass,
                    attachments,
                    gfx_hal::image::Extent {
                        width: self.dimensions.width,
                        height: self.dimensions.height,
                        depth: 1,
                    },
                )
                .unwrap()
        };

        unsafe {
            let fence = &self.submission_complete_fences[frame_idx];
            self.context
                .device
                .wait_for_fence(fence, !0)
                .expect("Failed to wait for fence");
            self.context
                .device
                .reset_fence(fence)
                .expect("Failed to reset fence");
            self.cmd_pools[frame_idx].reset(false);
        }

        // Rendering
        let cmd_buffer = &mut self.cmd_buffers[frame_idx];
        unsafe {
            cmd_buffer.begin_primary(gfx_hal::command::CommandBufferFlags::ONE_TIME_SUBMIT);

            cmd_buffer.set_viewports(
                0,
                &[Viewport {
                    rect: Rect {
                        x: self.viewport.rect.x,
                        y: self.viewport.rect.y + self.viewport.rect.h,
                        w: self.viewport.rect.w,
                        h: -self.viewport.rect.h,
                    },
                    depth: self.viewport.depth.clone(),
                }],
            );
            // TODO: Fix me!
            // cmd_buffer.set_scissors(0, &[self.viewport.rect]);

            let mut api = RendererApiCommon::new(
                &mut self.context,
                &self.static_context,
                &mut self.cmd_pools[frame_idx],
                cmd_buffer,
                &framebuffer,
                &self.viewport,
            );
            callback(&mut api);

            cmd_buffer.finish();

            let submission = Submission {
                command_buffers: iter::once(&*cmd_buffer),
                wait_semaphores: None,
                signal_semaphores: iter::once(&self.submission_complete_semaphores[frame_idx]),
            };
            self.context.queue_group.queues[0].submit(
                submission,
                Some(&self.submission_complete_fences[frame_idx]),
            );

            // present frame
            let result = self.context.queue_group.queues[0].present_surface(
                &mut self.surface,
                surface_image,
                Some(&self.submission_complete_semaphores[frame_idx]),
            );

            self.context.device.destroy_framebuffer(framebuffer);

            if result.is_err() {
                self.recreate_swapchain();
            }
        }

        // Increment our frame
        self.frame += 1;
    }
}

impl<B> Drop for Renderer<B>
where
    B: gfx_hal::Backend,
{
    fn drop(&mut self) {
        let result = self.context.device.wait_idle();
        assert!(result.is_ok(), "failed device to wait idle");

        unsafe {
            for p in self.cmd_pools.drain(..) {
                self.context.device.destroy_command_pool(p);
            }
            for s in self.submission_complete_semaphores.drain(..) {
                self.context.device.destroy_semaphore(s);
            }
            for f in self.submission_complete_fences.drain(..) {
                self.context.device.destroy_fence(f);
            }
            self.context
                .device
                .destroy_render_pass(ManuallyDrop::into_inner(std::ptr::read(
                    &self.context.render_pass,
                )));

            // Destroy depth resources
            self.context
                .device
                .destroy_image_view(ManuallyDrop::into_inner(std::ptr::read(
                    &self.depth_image_view,
                )));
            self.context
                .device
                .destroy_image(ManuallyDrop::into_inner(std::ptr::read(&self.depth_image)));
            self.context
                .device
                .free_memory(ManuallyDrop::into_inner(std::ptr::read(&self.depth_memory)));

            self.surface.unconfigure_swapchain(&self.context.device);
            if let Some(instance) = &self.instance {
                let surface = ManuallyDrop::into_inner(std::ptr::read(&self.surface));
                instance.destroy_surface(surface);
            }
        }
    }
}

pub struct RendererApiContext<B: Backend> {
    pub device: Rc<B::Device>,
    pub queue_group: QueueGroup<B>,
    pub first_render_pass: ManuallyDrop<B::RenderPass>,
    pub render_pass: ManuallyDrop<B::RenderPass>,
    pub use_first_render_pass: bool,
    pub clear_color: Vec4,
    pub clear_depth: f32,
}

pub struct RendererApiStaticContext {
    pub memory_types: Vec<MemoryType>,
    pub limits: Limits,
}

fn create_depth<B: Backend>(
    device: &B::Device,
    extent: gfx_hal::window::Extent2D,
    memory_types: &[MemoryType],
) -> (B::Image, B::Memory, B::ImageView, gfx_hal::format::Format) {
    let width = extent.width;
    let height = extent.height;
    let depth_stencil_format = gfx_hal::format::Format::D32SfloatS8Uint;
    let color_range = gfx_hal::image::SubresourceRange {
        aspects: gfx_hal::format::Aspects::COLOR,
        levels: 0..1,
        layers: 0..1,
    };
    let kind = gfx_hal::image::Kind::D2(
        width as gfx_hal::image::Size,
        height as gfx_hal::image::Size,
        1,
        1,
    );

    let mut image = unsafe {
        device.create_image(
            kind,
            1,
            depth_stencil_format,
            gfx_hal::image::Tiling::Optimal,
            gfx_hal::image::Usage::DEPTH_STENCIL_ATTACHMENT,
            gfx_hal::image::ViewCapabilities::empty(),
        )
    }
    .unwrap();
    let image_req = unsafe { device.get_image_requirements(&image) };

    let device_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, memory_type)| {
            image_req.type_mask & (1 << id) as u64 != 0
                && memory_type
                    .properties
                    .contains(gfx_hal::memory::Properties::DEVICE_LOCAL)
        })
        .unwrap()
        .into();
    let image_memory = unsafe { device.allocate_memory(device_type, image_req.size) }.unwrap();
    unsafe { device.bind_image_memory(&image_memory, 0, &mut image) }.unwrap();
    let image_view = unsafe {
        device.create_image_view(
            &image,
            gfx_hal::image::ViewKind::D2,
            depth_stencil_format,
            Swizzle::NO,
            color_range,
        )
    }
    .unwrap();

    (image, image_memory, image_view, depth_stencil_format)
}
