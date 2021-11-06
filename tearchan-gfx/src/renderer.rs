use crate::context::{GfxContext, GfxRenderContext};
use wgpu::{Features, Limits};
use winit::dpi::PhysicalSize;
use winit::window::Window;

#[allow(dead_code)]
pub struct Renderer {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    pub async fn new(
        window: &Window,
        optional_features: Features,
        required_features: Features,
        needed_limits: Limits,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| body.append_child(&WindowExtWebSys::canvas(window)).ok())
                .expect("couldn't append canvas to document body");
        }

        let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
        log::info!("backend: {:?}", backend);

        let instance = wgpu::Instance::new(backend);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(window);
            (size, surface)
        };
        let adapter =
            wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
                .await
                .expect("No suitable GPU adapters found on the system!");

        let adapter_features = adapter.features();
        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: (optional_features & adapter_features) | required_features,
                    limits: needed_limits,
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .expect("Unable to find a suitable GPU adapter!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&device, &surface_config);

        Renderer {
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
        }
    }

    pub fn create_context(&self) -> GfxContext {
        GfxContext {
            device: &self.device,
            queue: &self.queue,
            surface_config: &self.surface_config,
        }
    }

    pub fn create_surface_texture(&self) -> wgpu::SurfaceTexture {
        match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.surface_config);
                self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        }
    }

    pub fn create_render_context(
        &self,
        surface_texture: &wgpu::SurfaceTexture,
    ) -> (GfxContext, GfxRenderContext) {
        (
            self.create_context(),
            GfxRenderContext::new(surface_texture),
        )
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width.max(1);
        self.surface_config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }
}

pub struct RendererLazySetup {
    renderer: Option<Renderer>,
    window: Window,
}

impl RendererLazySetup {
    pub fn new(window: Window) -> Self {
        RendererLazySetup {
            renderer: None,
            window,
        }
    }

    pub async fn setup(
        &mut self,
        optional_features: Features,
        required_features: Features,
        needed_limits: Limits,
    ) {
        self.renderer = Some(
            Renderer::new(
                &self.window,
                optional_features,
                required_features,
                needed_limits,
            )
            .await,
        )
    }

    pub fn renderer_mut(&mut self) -> Option<&mut Renderer> {
        self.renderer.as_mut()
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
