use crate::hal::frame_resource::FrameResource;
use crate::hal::render_bundle::RenderBundle;
use crate::hal::render_pass::RenderPass;

pub struct RendererContextCommon<'a, B: gfx_hal::Backend> {
    render_bundle: &'a mut RenderBundle<B>,
    primary_render_pass: &'a mut RenderPass<B>,
    framebuffer: B::Framebuffer,
    frame_resource: &'a mut FrameResource<B>,
}

impl<'a, B: gfx_hal::Backend> RendererContextCommon<'a, B> {
    pub fn new(
        render_bundle: &'a mut RenderBundle<B>,
        primary_render_pass: &'a mut RenderPass<B>,
        framebuffer: B::Framebuffer,
        frame_resource: &'a mut FrameResource<B>,
    ) -> RendererContextCommon<'a, B> {
        RendererContextCommon {
            render_bundle,
            primary_render_pass,
            framebuffer,
            frame_resource,
        }
    }

    pub fn render_bundle(&self) -> &RenderBundle<B> {
        self.render_bundle
    }

    pub fn render_bundle_mut(&mut self) -> &mut RenderBundle<B> {
        self.render_bundle
    }
}
