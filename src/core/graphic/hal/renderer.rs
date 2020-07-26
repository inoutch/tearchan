use crate::core::graphic::hal::graphics::{GraphicsCommon, GraphicsContext};
use crate::core::graphic::hal::helper::{
    create_depth_resources, create_render_pass, find_queue_family,
};
use gfx_hal::adapter::{Adapter, PhysicalDevice};
use gfx_hal::command::{CommandBuffer, CommandBufferFlags, Level};
use gfx_hal::device::Device;
use gfx_hal::format::ChannelType;
use gfx_hal::image::Extent;
use gfx_hal::pool::{CommandPool, CommandPoolCreateFlags};
use gfx_hal::pso::{Rect, Viewport};
use gfx_hal::queue::{CommandQueue, Submission};
use gfx_hal::window::{Extent2D, PresentationSurface, Surface, SwapchainConfig};
use gfx_hal::Instance;
use nalgebra_glm::{vec2, vec4, Vec2};
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
    context: GraphicsContext<B>,
    actual_viewports: [Viewport; 1],
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
        let frames_in_flight = 1;

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
        let actual_viewports = [viewport_up_side_down(&viewport)];

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
            context: GraphicsContext {
                device: Rc::new(device),
                queue_group,
                render_pass,
                first_render_pass,
                use_first_render_pass: true,
                clear_color: vec4(0.0f32, 0.0f32, 0.0f32, 1.0f32),
                clear_depth: 0.0f32,
                memory_types,
                limits,
                display_size: DisplaySize {
                    logical: vec2(viewport.rect.w as _, viewport.rect.h as _),
                    physical: vec2(
                        default_dimensions.width as _,
                        default_dimensions.height as _,
                    ),
                    viewport,
                },
            },
            actual_viewports,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        let caps = self.surface.capabilities(&self.adapter.physical_device);
        let swap_config = SwapchainConfig::from_caps(
            &caps,
            self.surface_format,
            Extent2D {
                width: self.context.display_size.physical.x as _,
                height: self.context.display_size.physical.y as _,
            },
        );
        let extent = swap_config.extent;

        unsafe {
            self.surface
                .configure_swapchain(&self.context.device, swap_config)
                .expect("Can't create swapchain");
        }

        self.context.display_size.viewport.rect.w = extent.width as _;
        self.context.display_size.viewport.rect.h = extent.height as _;
        self.actual_viewports = [viewport_up_side_down(&self.context.display_size.viewport)];
        self.context.display_size.logical = vec2(extent.width as _, extent.height as _);

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
            create_depth_resources::<B>(&self.context.device, extent, &self.context.memory_types);

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
        F1: FnMut(&mut GraphicsCommon<B>),
        F2: FnMut(&mut ResizeContext),
    {
        self.context.use_first_render_pass = true;
        let frame_idx = self.frame as usize % self.frames_in_flight;
        let surface_image = unsafe {
            match self.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.recreate_swapchain();
                    let mut context = self.create_resize_context();
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
                    Extent {
                        width: self.context.display_size.physical.x as _,
                        height: self.context.display_size.physical.y as _,
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

            cmd_buffer.set_viewports(0, &self.actual_viewports);
            cmd_buffer.set_scissors(0, &[self.context.display_size.viewport.rect]);

            let mut api = GraphicsCommon::new(
                &mut self.context,
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
        self.context.display_size.physical = vec2(dimensions.width as _, dimensions.height as _);
    }

    pub fn create_resize_context(&self) -> ResizeContext {
        ResizeContext {
            display_size: &self.context.display_size,
        }
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
            for cmd_pool in self.cmd_pools.drain(..) {
                self.context.device.destroy_command_pool(cmd_pool);
            }
            for semaphore in self.submission_complete_semaphores.drain(..) {
                self.context.device.destroy_semaphore(semaphore);
            }
            for fence in self.submission_complete_fences.drain(..) {
                self.context.device.destroy_fence(fence);
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

#[derive(Clone)]
pub struct DisplaySize {
    pub logical: Vec2,
    pub physical: Vec2,
    pub viewport: Viewport,
}

pub struct ResizeContext<'a> {
    pub display_size: &'a DisplaySize,
}

fn viewport_up_side_down(viewport: &Viewport) -> Viewport {
    Viewport {
        rect: Rect {
            x: viewport.rect.x,
            y: viewport.rect.y + viewport.rect.h,
            w: viewport.rect.w,
            h: -viewport.rect.h,
        },
        depth: viewport.depth.clone(),
    }
}
