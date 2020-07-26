use crate::core::graphic::polygon::polygon_2d::{
    Polygon2DInterface, Polygon2DProvider, Polygon2DProviderInterface,
};
use crate::core::graphic::polygon::{Polygon, PolygonCore, PolygonProvider};
use crate::core::graphic::texture::TextureAtlas;
use crate::extension::shared::{clone_shared, make_shared, Shared};
use crate::math::mesh::square::{
    create_square_positions_from_frame, create_square_texcoords_from_frame,
};
use crate::math::mesh::MeshBuilder;
use nalgebra_glm::{vec2, Mat4, Vec2};
use std::collections::HashMap;

pub trait SpriteAtlasInterface {
    fn set_atlas(&mut self, index: usize);
    fn set_atlas_with_key(&mut self, key: &str);
}

pub struct SpriteAtlasProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    provider: T,
    texture_atlas: TextureAtlas,
    key_map: HashMap<String, usize>,
}

impl<T> SpriteAtlasProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    pub fn new(provider: T, texture_atlas: TextureAtlas) -> Self {
        let key_map: HashMap<_, _> = texture_atlas
            .frames
            .iter()
            .enumerate()
            .map(|(i, frame)| (frame.key.to_string(), i as usize))
            .collect();
        SpriteAtlasProvider {
            provider,
            texture_atlas,
            key_map,
        }
    }

    pub fn inner_provider(&self) -> &T {
        &self.provider
    }

    pub fn inner_provider_mut(&mut self) -> &mut T {
        &mut self.provider
    }
}

impl<T> PolygonProvider for SpriteAtlasProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    #[inline]
    fn transform(&self, core: &PolygonCore<Polygon>) -> Mat4 {
        self.provider.transform(core)
    }

    #[inline]
    fn transform_for_child(&self, core: &PolygonCore<Polygon>) -> Mat4 {
        self.provider.transform_for_child(core)
    }
}

impl<T> SpriteAtlasProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    pub fn set_atlas(&mut self, core: &mut PolygonCore, index: usize) {
        if let Some(frame) = self.texture_atlas.frames.get(index) {
            let positions = create_square_positions_from_frame(frame);
            let texcoords =
                create_square_texcoords_from_frame(self.texture_atlas.size.to_vec2(), frame);
            {
                self.provider.update_positions_of_mesh(core, positions);
                self.provider.update_texcoords_of_mesh(core, texcoords);
                self.provider.request_change(core);
            }
            self.provider
                .set_size(core, vec2(frame.source.w as f32, frame.source.h as f32));
        }
    }

    pub fn set_atlas_with_key(&mut self, core: &mut PolygonCore, key: &str) {
        if let Some(index) = self.key_map.get(key) {
            let i = *index;
            self.set_atlas(core, i);
        }
    }
}

pub struct SpriteAtlas {
    polygon: Shared<Polygon>,
    provider: Shared<SpriteAtlasProvider<Polygon2DProvider>>,
}

impl SpriteAtlas {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(texture_atlas: TextureAtlas) -> SpriteAtlas {
        let frame = texture_atlas
            .frames
            .first()
            .expect("There must be at least one or more frames");
        let mesh = MeshBuilder::new()
            .with_frame(texture_atlas.size.to_vec2(), frame)
            .build()
            .unwrap();

        let provider_2d =
            Polygon2DProvider::new(vec2(frame.source.w as f32, frame.source.h as f32));
        let provider = make_shared(SpriteAtlasProvider::new(provider_2d, texture_atlas));
        let cloned_provider = clone_shared(&provider);
        SpriteAtlas {
            polygon: make_shared(Polygon::new_with_provider(provider, mesh)),
            provider: cloned_provider,
        }
    }

    #[inline]
    pub fn polygon(&self) -> &Shared<Polygon> {
        &self.polygon
    }
}

impl SpriteAtlasInterface for SpriteAtlas {
    fn set_atlas(&mut self, index: usize) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .set_atlas(&mut polygon.core, index);
    }

    fn set_atlas_with_key(&mut self, key: &str) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .set_atlas_with_key(&mut polygon.core, key);
    }
}

impl Polygon2DInterface for SpriteAtlas {
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
            .set_anchor_point(&mut polygon.core, size);
    }

    fn size(&self) -> Vec2 {
        let polygon = self.polygon.borrow();
        self.provider.borrow_mut().inner_provider().size(&polygon.core).clone_owned()
    }
}
