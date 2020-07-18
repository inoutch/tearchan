use crate::core::graphic::polygon::polygon_2d::{
    Polygon2D, Polygon2DProvider, Polygon2DProviderInterface,
};
use crate::core::graphic::polygon::{Polygon, PolygonProvider};
use crate::core::graphic::texture::TextureAtlas;
use crate::extension::shared::Shared;
use crate::math::mesh::square::{
    create_square_positions_from_frame, create_square_texcoords_from_frame,
};
use crate::math::mesh::MeshBuilder;
use nalgebra_glm::vec2;
use serde::export::PhantomData;
use std::collections::HashMap;

pub type SpriteAtlas = SpriteAtlasCommon<Polygon2DProvider>;

pub struct SpriteAtlasCommon<T: 'static + Polygon2DProviderInterface> {
    polygon: Polygon2D,
    texture_atlas: TextureAtlas,
    key_map: HashMap<String, usize>,
    _provider: PhantomData<T>,
}

impl<T: 'static + Polygon2DProviderInterface> SpriteAtlasCommon<T> {
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
        let key_map: HashMap<_, _> = texture_atlas
            .frames
            .iter()
            .enumerate()
            .map(|(i, frame)| (frame.key.to_string(), i as usize))
            .collect();

        let polygon = Polygon2D::new(mesh, vec2(frame.source.w as f32, frame.source.h as f32));
        SpriteAtlas {
            texture_atlas,
            polygon,
            key_map,
            _provider: PhantomData,
        }
    }

    pub fn new_with_provider<U: Polygon2DProviderInterface>(
        provider: Box<dyn PolygonProvider>,
        texture_atlas: TextureAtlas,
    ) -> SpriteAtlasCommon<U> {
        let frame = texture_atlas
            .frames
            .first()
            .expect("There must be at least one or more frames");
        let mesh = MeshBuilder::new()
            .with_frame(texture_atlas.size.to_vec2(), frame)
            .build()
            .unwrap();
        let key_map: HashMap<_, _> = texture_atlas
            .frames
            .iter()
            .enumerate()
            .map(|(i, frame)| (frame.key.to_string(), i as usize))
            .collect();

        let polygon = Polygon2D::new_with_provider(provider, mesh);
        SpriteAtlasCommon {
            texture_atlas,
            polygon,
            key_map,
            _provider: PhantomData,
        }
    }

    pub fn set_atlas(&mut self, index: usize) {
        if let Some(frame) = self.texture_atlas.frames.get(index) {
            let positions = create_square_positions_from_frame(frame);
            let texcoords =
                create_square_texcoords_from_frame(self.texture_atlas.size.to_vec2(), frame);
            {
                let mut polygon = self.polygon.polygon().borrow_mut();
                polygon.core.update_positions_of_mesh(positions);
                polygon.core.update_texcoords_of_mesh(texcoords);
                polygon.core.request_change();
            }
            self.polygon
                .set_size::<T>(vec2(frame.source.w as f32, frame.source.h as f32));
        }
    }

    pub fn set_atlas_with_key(&mut self, key: &str) {
        if let Some(index) = self.key_map.get(key) {
            let i = *index;
            self.set_atlas(i);
        }
    }

    #[inline]
    pub fn polygon(&self) -> &Shared<Polygon> {
        &self.polygon.polygon()
    }

    #[inline]
    pub fn polygon2d(&self) -> &Polygon2D {
        &self.polygon
    }

    #[inline]
    pub fn polygon2d_mut(&mut self) -> &mut Polygon2D {
        &mut self.polygon
    }
}
