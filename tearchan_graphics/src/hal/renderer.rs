use crate::display_size::DisplaySize;
use crate::hal::backend::Backend;
use crate::hal::depth_resource::DepthResource;
use crate::hal::frame_resource::FrameResource;
use crate::hal::instance::create_backend;
use crate::hal::queue::find_queue_family;
use crate::hal::render_bundle::RenderBundleCommon;
use crate::hal::render_pass::RenderPass;
use crate::hal::renderer_context::RendererContextCommon;
use crate::hal::surface::find_support_format;
use crate::hal::viewport::convert_up_side_down;
use crate::screen::ScreenResolutionMode;
use gfx_hal::adapter::{Adapter, PhysicalDevice};
use gfx_hal::command::{CommandBuffer, CommandBufferFlags};
use gfx_hal::device::Device;
use gfx_hal::pass::AttachmentLoadOp;
use gfx_hal::pso::Viewport;
use gfx_hal::queue::{CommandQueue, Submission};
use gfx_hal::window::{Extent2D, PresentationSurface};
use gfx_hal::window::{Surface, SwapchainConfig};
use gfx_hal::Instance;
use nalgebra_glm::{vec2, TVec2, Vec2};
use std::borrow::Borrow;
use std::cell::Ref;
use std::iter;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::rc::Rc;
use tearchan_utility::shared::Shared;
use winit::window::Window;

pub enum RendererBeginResult<'a, B: gfx_hal::Backend> {
    Context {
        context: RendererContextCommon<'a, B>,
    },
    Resize,
}

#[derive(Clone, Debug)]
pub struct RendererProperties {
    pub frames_in_flight: u32,
}

impl Default for RendererProperties {
    fn default() -> Self {
        RendererProperties {
            frames_in_flight: 1,
        }
    }
}

pub struct Renderer<B: gfx_hal::Backend> {
    // For render instances
    instance: Option<B::Instance>,
    surface: ManuallyDrop<B::Surface>,
    adapter: Adapter<B>,
    depth_resource: DepthResource<B>,
    primary_render_pass: RenderPass<B>,
    frame_resources: Vec<FrameResource<B>>,
    // Private constants
    actual_viewports: [Viewport; 1],
    frame: u64,
    properties: RendererProperties,
    // Share resources
    render_bundle: RenderBundleCommon<B>,
    display_size: Shared<DisplaySize>,
}

impl<B> Renderer<B>
where
    B: gfx_hal::Backend,
{
    pub fn from_window(
        window: &Window,
        default_dimensions: TVec2<u32>,
        properties: RendererProperties,
    ) -> Renderer<Backend> {
        let (instance, mut adapters, surface) = create_backend(window);
        let adapter = adapters.remove(0);
        Renderer::new(instance, adapter, surface, default_dimensions, properties)
    }

    pub fn new(
        instance: Option<B::Instance>,
        adapter: Adapter<B>,
        mut surface: B::Surface,
        default_dimensions: TVec2<u32>,
        properties: RendererProperties,
    ) -> Renderer<B> {
        let physics_device = &adapter.physical_device;
        let memory_properties = physics_device.memory_properties();
        let limits = physics_device.limits();
        let family = find_queue_family(&adapter, &surface);
        let mut gpu = unsafe {
            physics_device
                .open(&[(family, &[1.0])], gfx_hal::Features::empty())
                .unwrap()
        };
        let queue_group = gpu.queue_groups.pop().unwrap();
        let device = gpu.device;
        let capabilities = surface.capabilities(&adapter.physical_device);
        let surface_format = find_support_format(&surface, &adapter);

        let swap_config = SwapchainConfig::from_caps(
            &capabilities,
            surface_format,
            Extent2D {
                width: default_dimensions.x,
                height: default_dimensions.y,
            },
        );
        let extent = swap_config.extent;
        unsafe {
            surface
                .configure_swapchain(&device, swap_config)
                .expect("Can't configure swapchain");
        };

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
        let actual_viewports = [convert_up_side_down(&viewport)];
        let display_size = Shared::new(DisplaySize {
            logical: vec2(viewport.rect.w as _, viewport.rect.h as _),
            physical: vec2(default_dimensions.x as _, default_dimensions.y as _),
            viewport,
        });
        let render_bundle = RenderBundleCommon::new(
            Rc::new(device),
            Shared::new(queue_group),
            Shared::clone(&display_size),
            Rc::new(memory_properties),
            Rc::new(limits),
        );
        let depth_resource = DepthResource::new(&render_bundle);
        let primary_render_pass = RenderPass::from_formats(
            &render_bundle,
            AttachmentLoadOp::Clear,
            AttachmentLoadOp::Clear,
            Some(surface_format),
            Some(depth_resource.image_resource().format().clone()),
        );

        let mut frame_resources = Vec::with_capacity(properties.frames_in_flight as _);
        for _ in 0..properties.frames_in_flight {
            frame_resources.push(FrameResource::new(&render_bundle));
        }

        Renderer {
            instance,
            surface: ManuallyDrop::new(surface),
            adapter,
            depth_resource,
            primary_render_pass,
            frame_resources,
            //
            actual_viewports,
            frame: 0u64,
            properties,
            //
            render_bundle,
            display_size,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        let display_size = self.render_bundle.display_size();
        let caps = self.surface.capabilities(&self.adapter.physical_device);
        let surface_format = find_support_format(self.surface.deref(), &self.adapter);
        let swap_config = SwapchainConfig::from_caps(
            &caps,
            surface_format,
            Extent2D {
                width: display_size.physical.x as _,
                height: display_size.physical.y as _,
            },
        );
        let extent = swap_config.extent;
        unsafe {
            self.surface
                .configure_swapchain(self.render_bundle.device(), swap_config)
                .expect("Can't create swapchain");
        }
        {
            let mut display_size = self.display_size.borrow_mut();
            display_size.viewport.rect.w = extent.width as _;
            display_size.viewport.rect.h = extent.height as _;
            display_size.logical = vec2(extent.width as _, extent.height as _);
            self.actual_viewports = [convert_up_side_down(&display_size.viewport)];
        }

        self.depth_resource = DepthResource::new(&self.render_bundle);
        self.primary_render_pass = RenderPass::from_formats(
            &self.render_bundle,
            AttachmentLoadOp::Clear,
            AttachmentLoadOp::Clear,
            Some(surface_format),
            Some(self.depth_resource.image_resource().format().clone()),
        );
    }

    pub fn render<F>(&mut self, scope: F)
    where
        F: FnOnce(RendererBeginResult<B>),
    {
        let frame_idx = (self.frame % self.properties.frames_in_flight as u64) as usize;
        let surface_image = unsafe {
            match self.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.recreate_swapchain();
                    return scope(RendererBeginResult::Resize);
                }
            }
        };

        let framebuffer_extent = {
            let size = self.display_size.borrow();
            vec2(size.physical.x as _, size.physical.y as _)
        };
        let mut framebuffer = self.primary_render_pass.create_framebuffer(
            vec![
                surface_image.borrow(),
                self.depth_resource.image_resource().image_view(),
            ],
            framebuffer_extent,
        );

        self.wait_fence(&mut self.frame_resources[frame_idx].fence());

        unsafe {
            let frame_resource = &mut self.frame_resources[frame_idx];
            let command_buffer = frame_resource.command_buffer_mut();
            command_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);
            command_buffer.set_viewports(0, &self.actual_viewports);
            command_buffer.set_scissors(0, &[self.display_size.borrow().viewport.rect]);
        }

        scope(RendererBeginResult::Context {
            context: RendererContextCommon::new(
                &mut self.render_bundle,
                &mut self.primary_render_pass,
                &mut framebuffer,
                &mut self.frame_resources[frame_idx],
            ),
        });

        unsafe {
            let frame_resource = &mut self.frame_resources[frame_idx];
            frame_resource.command_buffer_mut().finish();

            let command_buffer = frame_resource.command_buffer();

            // Present swapchain
            self.render_bundle.primary_command_queue_mut().submit(
                Submission {
                    command_buffers: iter::once(&*command_buffer),
                    wait_semaphores: None,
                    signal_semaphores: iter::once(frame_resource.submission_complete_semaphore()),
                },
                Some(frame_resource.submission_complete_fence()),
            );
            let result = self.render_bundle.primary_command_queue_mut().present(
                &mut self.surface,
                surface_image,
                Some(frame_resource.submission_complete_semaphore()),
            );

            self.render_bundle.device().destroy_framebuffer(framebuffer);
            self.render_bundle.device().wait_idle().unwrap();
            if result.is_err() {
                self.recreate_swapchain();
            }
        }

        // Increment our frame
        self.frame += 1;
    }

    pub fn set_dimensions(&mut self, dimensions: Vec2) {
        self.display_size.borrow_mut().physical = dimensions;
    }

    pub fn set_screen_resolution_mode(&mut self, resolution_mode: &ScreenResolutionMode) {
        self.display_size.borrow_mut().update(resolution_mode);
    }

    pub fn display_size(&self) -> Ref<DisplaySize> {
        self.display_size.borrow()
    }

    fn wait_fence(&self, fence: &B::Fence) {
        unsafe {
            self.render_bundle
                .device()
                .wait_for_fence(fence, !0)
                .expect("Failed to wait for fence");
            self.render_bundle
                .device()
                .reset_fence(fence)
                .expect("Failed to reset fence");
        }
    }
}

impl<B: gfx_hal::Backend> Drop for Renderer<B> {
    fn drop(&mut self) {
        if let Some(instance) = &self.instance {
            unsafe {
                let surface = ManuallyDrop::into_inner(std::ptr::read(&self.surface));
                instance.destroy_surface(surface);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::hal::renderer::{Renderer, RendererBeginResult, RendererProperties};
    use gfx_hal::Instance;
    use nalgebra_glm::vec2;
    use raw_window_handle::macos::MacOSHandle;
    use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

    struct DummyHasRawWindowHandle {}

    unsafe impl HasRawWindowHandle for DummyHasRawWindowHandle {
        fn raw_window_handle(&self) -> RawWindowHandle {
            RawWindowHandle::MacOS(MacOSHandle::empty())
        }
    }

    #[test]
    fn test() {
        let mut pass = false;

        let instance = gfx_backend_empty::Instance::create("test", 0u32).unwrap();
        let mut adapters = instance.enumerate_adapters();
        let surface = unsafe { instance.create_surface(&DummyHasRawWindowHandle {}) }.unwrap();
        let extent = vec2(200u32, 100u32);
        let properties = RendererProperties::default();
        let mut renderer: Renderer<gfx_backend_empty::Backend> =
            Renderer::new(None, adapters.remove(0), surface, extent, properties);

        renderer.render(|result| {
            match result {
                RendererBeginResult::Context { mut context } => {
                    // Queue command buffer
                    context.render_bundle_mut().primary_command_queue_mut();
                    context.render_bundle_mut().command_pool_mut();
                    context.render_bundle().device();
                    pass = true;
                }
                RendererBeginResult::Resize => {
                    // Resize
                }
            }
        });
        assert!(pass);
    }
}
