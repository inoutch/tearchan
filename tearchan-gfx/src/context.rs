pub struct GfxContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub surface_config: &'a wgpu::SurfaceConfiguration,
}

pub struct GfxRenderContext {
    pub view: wgpu::TextureView,
}

impl GfxRenderContext {
    pub fn new(frame: &wgpu::SurfaceFrame) -> Self {
        GfxRenderContext {
            view: frame
                .output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        }
    }
}
