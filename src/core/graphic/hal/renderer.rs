use gfx_hal::adapter::{Adapter, MemoryType, PhysicalDevice};
use gfx_hal::command::CommandBuffer;
use gfx_hal::device::Device;
use gfx_hal::format::ChannelType;
use gfx_hal::pool::CommandPool;
use gfx_hal::queue::{CommandQueue, QueueFamily, QueueGroup, Submission};
use gfx_hal::window::{PresentationSurface, Surface};
use gfx_hal::{window, Instance, Limits};
use std::borrow::Borrow;
use std::iter;
use std::mem::ManuallyDrop;

use crate::core::graphic::hal::renderer_api::Api;
use nalgebra_glm::vec2;
use std::ops::Deref;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

#[cfg(feature = "dx11")]
extern crate gfx_backend_dx11 as back;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(any(feature = "gl"))]
extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;

const DIMS: window::Extent2D = window::Extent2D {
    width: 1024,
    height: 768,
};

pub struct Renderer<B: gfx_hal::Backend> {
    instance: Option<B::Instance>,
    surface: ManuallyDrop<B::Surface>,
    adapter: Adapter<B>,
    device: Rc<B::Device>,
    queue_group: QueueGroup<B>,
    surface_format: gfx_hal::format::Format,
    dimensions: window::Extent2D,
    viewport: gfx_hal::pso::Viewport,
    cmd_pools: Vec<B::CommandPool>,
    cmd_buffers: Vec<B::CommandBuffer>,
    render_pass: ManuallyDrop<B::RenderPass>,
    submission_complete_semaphores: Vec<B::Semaphore>,
    submission_complete_fences: Vec<B::Fence>,
    frames_in_flight: usize,
    frame: u64,
    memory_types: Vec<MemoryType>,
    limits: Limits,
}

impl<B> Renderer<B>
where
    B: gfx_hal::Backend,
{
    pub fn new(
        instance: Option<B::Instance>,
        adapter: Adapter<B>,
        mut surface: B::Surface,
    ) -> Renderer<B> {
        let memory_types = adapter.physical_device.memory_properties().memory_types;
        let limits = adapter.physical_device.limits();

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
        println!("formats: {:?}", formats);
        let surface_format = formats.map_or(gfx_hal::format::Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .copied()
                .unwrap_or(formats[0])
        });

        let swap_config = window::SwapchainConfig::from_caps(&caps, surface_format, DIMS);
        println!("{:?}", swap_config);
        let extent = swap_config.extent;
        unsafe {
            surface
                .configure_swapchain(&device, swap_config)
                .expect("Can't configure swapchain");
        };

        let render_pass = {
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

            let subpass = gfx_hal::pass::SubpassDesc {
                colors: &[(0, gfx_hal::image::Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            ManuallyDrop::new(
                unsafe { device.create_render_pass(&[attachment], &[subpass], &[]) }
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

        for i in 0..frames_in_flight {
            submission_complete_semaphores.push(
                device
                    .create_semaphore()
                    .expect("Could not create semaphore"),
            );
            submission_complete_fences
                .push(device.create_fence(true).expect("Could not create fence"));
            cmd_buffers
                .push(unsafe { cmd_pools[i].allocate_one(gfx_hal::command::Level::Primary) });
        }

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
        println!("viewport {:?}", viewport);

        Renderer {
            instance,
            surface: ManuallyDrop::new(surface),
            adapter,
            device: Rc::new(device),
            queue_group,
            surface_format,
            dimensions: DIMS,
            viewport,
            render_pass,
            frames_in_flight,
            submission_complete_semaphores,
            submission_complete_fences,
            cmd_pools,
            cmd_buffers,
            frame: 0,
            memory_types,
            limits,
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
                .configure_swapchain(&self.device, swap_config)
                .expect("Can't create swapchain");
        }

        self.viewport.rect.w = extent.width as _;
        self.viewport.rect.h = extent.height as _;
    }

    pub fn render<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Api<B>) -> (),
    {
        let surface_image = unsafe {
            match self.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.recreate_swapchain();
                    return;
                }
            }
        };

        let framebuffer = unsafe {
            self.device
                .create_framebuffer(
                    &self.render_pass,
                    iter::once(surface_image.borrow()),
                    gfx_hal::image::Extent {
                        width: self.dimensions.width,
                        height: self.dimensions.height,
                        depth: 1,
                    },
                )
                .unwrap()
        };

        let frame_idx = self.frame as usize % self.frames_in_flight;

        unsafe {
            let fence = &self.submission_complete_fences[frame_idx];
            self.device
                .wait_for_fence(fence, !0)
                .expect("Failed to wait for fence");
            self.device
                .reset_fence(fence)
                .expect("Failed to reset fence");
            self.cmd_pools[frame_idx].reset(false);
        }

        // Rendering
        let cmd_buffer = &mut self.cmd_buffers[frame_idx];
        unsafe {
            cmd_buffer.begin_primary(gfx_hal::command::CommandBufferFlags::ONE_TIME_SUBMIT);

            // cmd_buffer.set_viewports(0, &[self.viewport.clone()]);
            // cmd_buffer.set_scissors(0, &[self.viewport.rect]);

            // TODO: Group arguments the ungenerated items in each loop
            let screen_size = vec2(self.viewport.rect.w as f32, self.viewport.rect.h as f32);
            let mut api = Api::new(
                &self.device,
                &self.limits,
                &self.memory_types,
                &mut self.cmd_pools[frame_idx],
                cmd_buffer,
                &mut self.queue_group,
                self.render_pass.deref(),
                &framebuffer,
                &self.viewport,
                &screen_size,
            );
            callback(&mut api);

            cmd_buffer.finish();

            let submission = Submission {
                command_buffers: iter::once(&*cmd_buffer),
                wait_semaphores: None,
                signal_semaphores: iter::once(&self.submission_complete_semaphores[frame_idx]),
            };
            self.queue_group.queues[0].submit(
                submission,
                Some(&self.submission_complete_fences[frame_idx]),
            );

            // present frame
            let result = self.queue_group.queues[0].present_surface(
                &mut self.surface,
                surface_image,
                Some(&self.submission_complete_semaphores[frame_idx]),
            );

            self.device.destroy_framebuffer(framebuffer);

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
        let result = self.device.wait_idle();
        assert!(result.is_ok(), "failed device to wait idle");

        unsafe {
            for p in self.cmd_pools.drain(..) {
                self.device.destroy_command_pool(p);
            }
            for s in self.submission_complete_semaphores.drain(..) {
                self.device.destroy_semaphore(s);
            }
            for f in self.submission_complete_fences.drain(..) {
                self.device.destroy_fence(f);
            }
            self.device
                .destroy_render_pass(ManuallyDrop::into_inner(std::ptr::read(&self.render_pass)));

            self.surface.unconfigure_swapchain(&self.device);
            if let Some(instance) = &self.instance {
                let surface = ManuallyDrop::into_inner(std::ptr::read(&self.surface));
                instance.destroy_surface(surface);
            }
        }
    }
}
