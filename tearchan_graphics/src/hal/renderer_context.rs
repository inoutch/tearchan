use crate::hal::frame_resource::FrameResource;
use crate::hal::render_bundle::RenderBundleCommon;
use crate::hal::render_pass::RenderPass;

pub struct RendererContextCommon<'a, B: gfx_hal::Backend> {
    render_bundle: &'a mut RenderBundleCommon<B>,
    primary_render_pass: &'a mut RenderPass<B>,
    framebuffer: &'a mut B::Framebuffer,
    frame_resource: &'a mut FrameResource<B>,
}

impl<'a, B: gfx_hal::Backend> RendererContextCommon<'a, B> {
    pub fn new(
        render_bundle: &'a mut RenderBundleCommon<B>,
        primary_render_pass: &'a mut RenderPass<B>,
        framebuffer: &'a mut B::Framebuffer,
        frame_resource: &'a mut FrameResource<B>,
    ) -> RendererContextCommon<'a, B> {
        RendererContextCommon {
            render_bundle,
            primary_render_pass,
            framebuffer,
            frame_resource,
        }
    }

    pub fn render_bundle(&self) -> &RenderBundleCommon<B> {
        self.render_bundle
    }

    pub fn render_bundle_mut(&mut self) -> &mut RenderBundleCommon<B> {
        self.render_bundle
    }

    pub fn primary_render_pass(&self) -> &RenderPass<B> {
        self.primary_render_pass
    }
}
