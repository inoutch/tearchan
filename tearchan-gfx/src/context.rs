pub struct GfxContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub surface_config: &'a wgpu::SurfaceConfiguration,
}

pub struct GfxRenderContext {
    pub view: wgpu::TextureView,
}

impl GfxRenderContext {
    pub fn new(surface_texture: &wgpu::SurfaceTexture) -> Self {
        GfxRenderContext {
            view: surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        }
    }
}
