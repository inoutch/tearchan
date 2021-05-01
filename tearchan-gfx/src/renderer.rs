use crate::context::{GfxContext, GfxRenderContext};
use wgpu::{Features, Limits};
use winit::dpi::PhysicalSize;
use winit::window::Window;

#[allow(dead_code)]
pub struct Renderer {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain: wgpu::SwapChain,
    swapchain_desc: wgpu::SwapChainDescriptor,
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

        let backend = if let Ok(backend) = std::env::var("WGPU_BACKEND") {
            match backend.to_lowercase().as_str() {
                "vulkan" => wgpu::BackendBit::VULKAN,
                "metal" => wgpu::BackendBit::METAL,
                "dx12" => wgpu::BackendBit::DX12,
                "dx11" => wgpu::BackendBit::DX11,
                "gl" => wgpu::BackendBit::GL,
                "webgpu" => wgpu::BackendBit::BROWSER_WEBGPU,
                other => panic!("Unknown backend: {}", other),
            }
        } else if cfg!(target_arch = "wasm32") {
            wgpu::BackendBit::GL
        } else {
            wgpu::BackendBit::PRIMARY
        };
        log::info!("backend: {:?}", backend);
        let power_preference = if let Ok(power_preference) = std::env::var("WGPU_POWER_PREF") {
            match power_preference.to_lowercase().as_str() {
                "low" => wgpu::PowerPreference::LowPower,
                "high" => wgpu::PowerPreference::HighPerformance,
                other => panic!("Unknown power preference: {}", other),
            }
        } else {
            wgpu::PowerPreference::default()
        };
        let instance = wgpu::Instance::new(backend);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(window);
            (size, surface)
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(&surface),
            })
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

        let swapchain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swapchain = device.create_swap_chain(&surface, &swapchain_desc);

        Renderer {
            instance,
            surface,
            adapter,
            device,
            queue,
            swapchain,
            swapchain_desc,
        }
    }

    pub fn create_context(&self) -> GfxContext {
        GfxContext {
            device: &self.device,
            queue: &self.queue,
            swapchain_desc: &self.swapchain_desc,
        }
    }

    pub fn create_render_context(&mut self) -> (GfxContext, GfxRenderContext) {
        let frame = match self.swapchain.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                self.swapchain = self
                    .device
                    .create_swap_chain(&self.surface, &self.swapchain_desc);
                self.swapchain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture!")
            }
        };
        (self.create_context(), GfxRenderContext::new(frame))
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.swapchain_desc.width = if size.width == 0 { 1 } else { size.width };
        self.swapchain_desc.height = if size.height == 0 { 1 } else { size.height };
        self.swapchain = self
            .device
            .create_swap_chain(&self.surface, &self.swapchain_desc);
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
