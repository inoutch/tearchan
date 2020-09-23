use crate::hal::render_bundle::RenderBundleCommon;
use gfx_hal::device::Device;
use gfx_hal::format::Format;
use gfx_hal::image::{Extent, Layout};
use gfx_hal::pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp};
use gfx_hal::Backend;
use nalgebra_glm::TVec2;
use std::mem::ManuallyDrop;

pub struct RenderPass<B: Backend> {
    render_bundle: RenderBundleCommon<B>,
    render_pass: ManuallyDrop<B::RenderPass>,
}

impl<B: Backend> RenderPass<B> {
    pub fn from_formats(
        render_bundle: &RenderBundleCommon<B>,
        color_load_op: AttachmentLoadOp,
        depth_load_op: AttachmentLoadOp,
        color_format: Option<Format>,
        depth_stencil_format: Option<Format>,
    ) -> RenderPass<B> {
        let attachment = Attachment {
            format: color_format,
            samples: 1,
            ops: AttachmentOps::new(color_load_op, AttachmentStoreOp::Store),
            stencil_ops: AttachmentOps::DONT_CARE,
            layouts: Layout::Undefined..Layout::Present,
        };
        let depth_attachment = Attachment {
            format: depth_stencil_format,
            samples: 1,
            ops: AttachmentOps::new(depth_load_op, AttachmentStoreOp::Store),
            stencil_ops: AttachmentOps::DONT_CARE,
            layouts: Layout::Undefined..Layout::DepthStencilAttachmentOptimal,
        };
        let subpass = gfx_hal::pass::SubpassDesc {
            colors: &[(0, gfx_hal::image::Layout::ColorAttachmentOptimal)],
            depth_stencil: Some(&(1, gfx_hal::image::Layout::DepthStencilAttachmentOptimal)),
            inputs: &[],
            resolves: &[],
            preserves: &[],
        };

        let render_pass = unsafe {
            render_bundle.device().create_render_pass(
                &[attachment, depth_attachment],
                &[subpass],
                &[],
            )
        }
        .unwrap();
        RenderPass {
            render_bundle: render_bundle.clone(),
            render_pass: ManuallyDrop::new(render_pass),
        }
    }

    pub fn create_framebuffer(
        &self,
        image_views: Vec<&B::ImageView>,
        extent: TVec2<u32>,
    ) -> B::Framebuffer {
        let extent = Extent {
            width: extent.x,
            height: extent.y,
            depth: 1,
        };
        unsafe {
            self.render_bundle
                .device()
                .create_framebuffer(&self.render_pass, image_views, extent)
        }
        .unwrap()
    }

    pub fn get(&self) -> &B::RenderPass {
        &self.render_pass
    }
}

impl<B: Backend> Drop for RenderPass<B> {
    fn drop(&mut self) {
        unsafe {
            self.render_bundle
                .device()
                .destroy_render_pass(ManuallyDrop::into_inner(std::ptr::read(&self.render_pass)))
        }
    }
}
