use crate::hal::image_resource::ImageResource;
use crate::hal::render_bundle::RenderBundleCommon;
use crate::image::Image;
use gfx_hal::device::Device;
use gfx_hal::image::{Filter, SamplerDesc, WrapMode};
use gfx_hal::Backend;
use nalgebra_glm::vec2;
use std::mem::ManuallyDrop;

#[derive(Clone)]
pub struct TextureConfig {
    pub filter: Filter,
    pub wrap: WrapMode,
}

impl TextureConfig {
    pub fn for_pixel() -> Self {
        TextureConfig {
            filter: Filter::Nearest,
            wrap: WrapMode::Clamp,
        }
    }
}

impl Default for TextureConfig {
    fn default() -> Self {
        TextureConfig {
            filter: Filter::Linear,
            wrap: WrapMode::Clamp,
        }
    }
}

pub struct TextureCommon<B: Backend> {
    render_bundle: RenderBundleCommon<B>,
    image_resource: ImageResource<B>,
    sampler: ManuallyDrop<B::Sampler>,
}

impl<B: Backend> TextureCommon<B> {
    pub fn new(
        render_bundle: &RenderBundleCommon<B>,
        image: &Image,
        config: TextureConfig,
    ) -> TextureCommon<B> {
        let mut image_resource =
            ImageResource::new_for_texture(render_bundle, image.size().clone_owned());
        let _ = image_resource.copy(image, vec2(0u32, 0u32)).unwrap();

        let sampler = create_sampler(&render_bundle, config);

        TextureCommon {
            render_bundle: render_bundle.clone(),
            image_resource,
            sampler,
        }
    }

    pub fn image_resource(&self) -> &ImageResource<B> {
        &self.image_resource
    }

    pub fn set_config(&mut self, config: TextureConfig) {
        destroy_sampler(&self.render_bundle, &self.sampler);
        self.sampler = create_sampler(&self.render_bundle, config);
    }
}

impl<B: Backend> Drop for TextureCommon<B> {
    fn drop(&mut self) {
        destroy_sampler(&self.render_bundle, &self.sampler);
    }
}

fn create_sampler<B: Backend>(
    render_bundle: &RenderBundleCommon<B>,
    config: TextureConfig,
) -> ManuallyDrop<B::Sampler> {
    ManuallyDrop::new(
        unsafe {
            render_bundle
                .device()
                .create_sampler(&SamplerDesc::new(config.filter, config.wrap))
        }
        .expect("Can't create sampler"),
    )
}

fn destroy_sampler<B: Backend>(
    render_bundle: &RenderBundleCommon<B>,
    sampler: &ManuallyDrop<B::Sampler>,
) {
    unsafe {
        render_bundle
            .device()
            .destroy_sampler(ManuallyDrop::into_inner(std::ptr::read(sampler)));
    }
}
