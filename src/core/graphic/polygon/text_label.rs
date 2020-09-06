use crate::core::graphic::font_texture::FontTexture;
use crate::core::graphic::polygon::polygon_2d::{
    Polygon2DInterface, Polygon2DProvider, Polygon2DProviderInterface,
};
use crate::core::graphic::polygon::{Polygon, PolygonCore, PolygonProvider};
use crate::extension::shared::{clone_shared, make_shared, Shared};
use nalgebra_glm::{Mat4, Vec2};

pub struct TextLabelProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    provider: T,
}

impl<T> TextLabelProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    pub fn new(provider: T) -> Self {
        TextLabelProvider { provider }
    }

    pub fn inner_provider(&self) -> &T {
        &self.provider
    }

    pub fn inner_provider_mut(&mut self) -> &mut T {
        &mut self.provider
    }
}

impl<T> PolygonProvider for TextLabelProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    fn transform(&self, core: &PolygonCore<Polygon>) -> Mat4 {
        self.provider.transform(core)
    }

    fn transform_for_child(&self, core: &PolygonCore<Polygon>) -> Mat4 {
        self.provider.transform_for_child(core)
    }
}

pub struct TextLabel {
    polygon: Shared<Polygon>,
    provider: Shared<TextLabelProvider<Polygon2DProvider>>,
}

impl TextLabel {
    pub fn new(font_texture: &FontTexture) -> Self {
        let (mesh, size) = font_texture.build_mesh();
        let provider_2d = Polygon2DProvider::new(size);
        let provider = make_shared(TextLabelProvider::new(provider_2d));
        let cloned_provider = clone_shared(&provider);
        TextLabel {
            polygon: make_shared(Polygon::new_with_provider(provider, mesh)),
            provider: cloned_provider,
        }
    }

    pub fn polygon(&self) -> &Shared<Polygon> {
        &self.polygon
    }
}

impl Polygon2DInterface for TextLabel {
    fn set_anchor_point(&mut self, anchor_point: Vec2) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .inner_provider_mut()
            .set_anchor_point(&mut polygon.core, anchor_point);
    }

    fn set_size(&mut self, size: Vec2) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .inner_provider_mut()
            .set_size(&mut polygon.core, size);
    }

    fn size(&self) -> Vec2 {
        let polygon = self.polygon.borrow();
        self.provider.borrow().inner_provider().size(&polygon.core).clone_owned()
    }
}
