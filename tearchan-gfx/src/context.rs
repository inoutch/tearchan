pub struct GfxContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub swapchain_desc: &'a wgpu::SwapChainDescriptor,
}

pub struct GfxRenderContext {
    frame: wgpu::SwapChainFrame,
}

impl GfxRenderContext {
    pub fn new(frame: wgpu::SwapChainFrame) -> Self {
        GfxRenderContext { frame }
    }

    pub fn frame(&self) -> &wgpu::SwapChainTexture {
        &self.frame.output
    }
}
