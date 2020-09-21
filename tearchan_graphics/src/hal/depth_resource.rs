use crate::hal::image_resource::ImageResource;
use crate::hal::render_bundle::RenderBundleCommon;
use gfx_hal::Backend;
use nalgebra_glm::vec2;

pub struct DepthResource<B: Backend> {
    image_resource: ImageResource<B>,
}

impl<B: Backend> DepthResource<B> {
    pub fn new(render_bundle: &RenderBundleCommon<B>) -> DepthResource<B> {
        let extent = vec2(
            render_bundle.display_size().physical.x as u32,
            render_bundle.display_size().physical.y as u32,
        );
        DepthResource {
            image_resource: ImageResource::new_for_depth(render_bundle, extent),
        }
    }

    pub fn image_resource(&self) -> &ImageResource<B> {
        &self.image_resource
    }
}
