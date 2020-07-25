use crate::core::graphic::hal::renderer_api::RendererApiCommon;
use gfx_hal::adapter::{Adapter, MemoryType, PhysicalDevice};
use gfx_hal::command::{CommandBuffer, CommandBufferFlags, Level};
use gfx_hal::device::Device;
use gfx_hal::format::{ChannelType, Format, Swizzle};
use gfx_hal::image::Layout;
use gfx_hal::pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp};
use gfx_hal::pool::{CommandPool, CommandPoolCreateFlags};
use gfx_hal::pso::{Rect, Viewport};
use gfx_hal::queue::{CommandQueue, QueueFamily, QueueGroup, Submission};
use gfx_hal::window::{Extent2D, PresentationSurface, Surface, SwapchainConfig};
use gfx_hal::{Backend, Instance, Limits};
use nalgebra_glm::{vec2, vec4, Vec2, Vec4};
use std::borrow::Borrow;
use std::iter;
use std::mem::ManuallyDrop;
use std::rc::Rc;

pub struct Renderer<B: gfx_hal::Backend> {
    // private gfx-hal instances
    instance: Option<B::Instance>,
    surface: ManuallyDrop<B::Surface>,
    adapter: Adapter<B>,
    cmd_pools: Vec<B::CommandPool>,
    cmd_buffers: Vec<B::CommandBuffer>,
    submission_complete_semaphores: Vec<B::Semaphore>,
    submission_complete_fences: Vec<B::Fence>,
    depth_image: ManuallyDrop<B::Image>,
    depth_memory: ManuallyDrop<B::Memory>,
    depth_image_view: ManuallyDrop<B::ImageView>,
    // private gfx-hal properties
    surface_format: gfx_hal::format::Format,
    frames_in_flight: usize,
    frame: u64,
    // public properties
    context: RendererApiContext<B>,
    properties: RendererApiProperties,
}

impl<B> Renderer<B>
where
    B: gfx_hal::Backend,
{
    pub fn new(
        instance: Option<B::Instance>,
        adapter: Adapter<B>,
        mut surface: B::Surface,
        default_dimensions: Extent2D,
    ) -> Renderer<B> {
        let memory_types = adapter.physical_device.memory_properties().memory_types;
        let limits = adapter.physical_device.limits();

        // Build a new device and associated command queues
        let family = find_queue_family(&adapter, &surface);
        let mut gpu = unsafe {
            adapter
                .physical_device
                .open(&[(family, &[1.0])], gfx_hal::Features::empty())
                .unwrap()
        };
        let queue_group = gpu.queue_groups.pop().unwrap();
        let device = gpu.device;

        let caps = surface.capabilities(&adapter.physical_device);
        let formats = surface.supported_formats(&adapter.physical_device);
        let surface_format = formats.map_or(gfx_hal::format::Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .copied()
                .unwrap_or(formats[0])
        });

        let swap_config = SwapchainConfig::from_caps(&caps, surface_format, default_dimensions);
        let extent = swap_config.extent;
        unsafe {
            surface
                .configure_swapchain(&device, swap_config)
                .expect("Can't configure swapchain");
        };

        let (depth_image, depth_memory, depth_image_view, depth_stencil_format) =
            create_depth_resources::<B>(&device, extent, &memory_types);

        let first_render_pass = ManuallyDrop::new(create_render_pass::<B>(
            &device,
            &surface_format,
            &depth_stencil_format,
            true,
        ));

        let render_pass = ManuallyDrop::new(create_render_pass::<B>(
            &device,
            &surface_format,
            &depth_stencil_format,
            false,
        ));

        let frames_in_flight = 1;

        let mut submission_complete_semaphores = Vec::with_capacity(frames_in_flight);
        let mut submission_complete_fences = Vec::with_capacity(frames_in_flight);
        let mut cmd_pools = Vec::with_capacity(frames_in_flight);
        let mut cmd_buffers = Vec::with_capacity(frames_in_flight);

        for _ in 0..frames_in_flight {
            let cmd_pool = unsafe {
                device
                    .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
                    .expect("Can't create command pool")
            };
            cmd_pools.push(cmd_pool);
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
                cmd_buffers.push(unsafe { cmd_pool.allocate_one(Level::Primary) });
            });

        // Rendering setup
        let viewport = Viewport {
            rect: gfx_hal::pso::Rect {
                x: 0,
                y: 0,
                w: extent.width as _,
                h: extent.height as _,
            },
            depth: 0.0..1.0,
        };

        let properties = RendererApiProperties {
            memory_types,
            limits,
            display_size: DisplaySize {
                screen: vec2(viewport.rect.w as _, viewport.rect.h as _),
                device: vec2(
                    default_dimensions.width as _,
                    default_dimensions.height as _,
                ),
                viewport,
            },
        };

        Renderer {
            // private gfx-hal instances
            instance,
            surface: ManuallyDrop::new(surface),
            adapter,
            cmd_pools,
            cmd_buffers,
            submission_complete_semaphores,
            submission_complete_fences,
            depth_image: ManuallyDrop::new(depth_image),
            depth_memory: ManuallyDrop::new(depth_memory),
            depth_image_view: ManuallyDrop::new(depth_image_view),
            // private gfx-hal properties
            surface_format,
            frames_in_flight,
            frame: 0,
            // public properties
            context: RendererApiContext {
                device: Rc::new(device),
                queue_group,
                render_pass,
                first_render_pass,
                use_first_render_pass: true,
                clear_color: vec4(0.0f32, 0.0f32, 0.0f32, 1.0f32),
                clear_depth: 0.0f32,
            },
            properties,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        let caps = self.surface.capabilities(&self.adapter.physical_device);
        let swap_config = SwapchainConfig::from_caps(
            &caps,
            self.surface_format,
            Extent2D {
                width: self.properties.display_size.device.x as _,
                height: self.properties.display_size.device.y as _,
            },
        );
        let extent = swap_config.extent;

        unsafe {
            self.surface
                .configure_swapchain(&self.context.device, swap_config)
                .expect("Can't create swapchain");
        }

        self.properties.display_size.viewport.rect.w = extent.width as _;
        self.properties.display_size.viewport.rect.h = extent.height as _;

        unsafe {
            self.context
                .device
                .destroy_render_pass(ManuallyDrop::into_inner(std::ptr::read(
                    &self.context.render_pass,
                )));
            self.context
                .device
                .destroy_render_pass(ManuallyDrop::into_inner(std::ptr::read(
                    &self.context.first_render_pass,
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
        }

        let (depth_image, depth_memory, depth_image_view, depth_stencil_format) =
            create_depth_resources::<B>(
                &self.context.device,
                extent,
                &self.properties.memory_types,
            );

        let first_render_pass = ManuallyDrop::new(create_render_pass::<B>(
            &self.context.device,
            &self.surface_format,
            &depth_stencil_format,
            true,
        ));

        let render_pass = ManuallyDrop::new(create_render_pass::<B>(
            &self.context.device,
            &self.surface_format,
            &depth_stencil_format,
            false,
        ));

        self.context.render_pass = render_pass;
        self.context.first_render_pass = first_render_pass;
        self.depth_image = ManuallyDrop::new(depth_image);
        self.depth_memory = ManuallyDrop::new(depth_memory);
        self.depth_image_view = ManuallyDrop::new(depth_image_view);
    }

    pub fn render<F1, F2>(&mut self, mut render_callback: F1, mut resize_callback: F2)
    where
        F1: FnMut(&mut RendererApiCommon<B>),
        F2: FnMut(&mut ResizeContext),
    {
        self.context.use_first_render_pass = true;

        let frame_idx = self.frame as usize % self.frames_in_flight;
        let surface_image = unsafe {
            match self.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.recreate_swapchain();
                    let mut context = ResizeContext {
                        display_size: &self.properties.display_size,
                    };
                    resize_callback(&mut context);
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
                        width: self.properties.display_size.device.x as _,
                        height: self.properties.display_size.device.y as _,
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
            cmd_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);

            cmd_buffer.set_viewports(
                0,
                &[Viewport {
                    rect: Rect {
                        x: self.properties.display_size.viewport.rect.x,
                        y: self.properties.display_size.viewport.rect.y
                            + self.properties.display_size.viewport.rect.h,
                        w: self.properties.display_size.viewport.rect.w,
                        h: -self.properties.display_size.viewport.rect.h,
                    },
                    depth: self.properties.display_size.viewport.depth.clone(),
                }],
            );
            cmd_buffer.set_scissors(0, &[self.properties.display_size.viewport.rect]);

            let mut api = RendererApiCommon::new(
                &mut self.context,
                &self.properties,
                &mut self.cmd_pools[frame_idx],
                cmd_buffer,
                &framebuffer,
            );
            render_callback(&mut api);

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

    pub fn set_dimensions(&mut self, dimensions: Extent2D) {
        self.properties.display_size.device = vec2(dimensions.width as _, dimensions.height as _);
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
            self.context
                .device
                .destroy_render_pass(ManuallyDrop::into_inner(std::ptr::read(
                    &self.context.first_render_pass,
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

pub struct RendererApiProperties {
    pub memory_types: Vec<MemoryType>,
    pub limits: Limits,
    pub display_size: DisplaySize,
}

fn create_depth_resources<B: Backend>(
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

fn find_queue_family<'a, B: Backend>(
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

fn create_render_pass<B: Backend>(
    device: &B::Device,
    surface_format: &Format,
    depth_stencil_format: &Format,
    first: bool,
) -> B::RenderPass {
    let load_op = if first {
        AttachmentLoadOp::Clear
    } else {
        AttachmentLoadOp::Load
    };

    let attachment = Attachment {
        format: Some(*surface_format),
        samples: 1,
        ops: AttachmentOps::new(load_op, AttachmentStoreOp::Store),
        stencil_ops: AttachmentOps::DONT_CARE,
        layouts: Layout::Undefined..Layout::Present,
    };
    let depth_attachment = Attachment {
        format: Some(*depth_stencil_format),
        samples: 1,
        ops: AttachmentOps::new(load_op, AttachmentStoreOp::Store),
        stencil_ops: AttachmentOps::DONT_CARE,
        layouts: Layout::Undefined..Layout::DepthStencilAttachmentOptimal,
    };
    let subpass = gfx_hal::pass::SubpassDesc {
        colors: &[(0, gfx_hal::image::Layout::ColorAttachmentOptimal)],
        depth_stencil: Some(&(1, gfx_hal::image::Layout::DepthStencilAttachmentOptimal)),
        inputs: &[],
        resolves: &[],
        preserves: &[],
    };

    unsafe { device.create_render_pass(&[attachment, depth_attachment], &[subpass], &[]) }.unwrap()
}

pub struct DisplaySize {
    pub screen: Vec2,
    pub device: Vec2,
    pub viewport: Viewport,
}

pub struct ResizeContext<'a> {
    pub display_size: &'a DisplaySize,
}
