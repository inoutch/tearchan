use crate::core::graphic::polygon::polygon_2d::Polygon2D;
use crate::core::graphic::polygon::{Polygon, PolygonProvider};
use crate::core::graphic::texture::TextureAtlas;
use crate::extension::shared::Shared;
use crate::math::mesh::{
    create_square_positions_from_frame, create_square_texcoords_from_frame, MeshBuilder,
};
use nalgebra_glm::vec2;
use std::collections::HashMap;

pub struct SpriteAtlas {
    polygon: Polygon2D,
    texture_atlas: TextureAtlas,
    key_map: HashMap<String, usize>,
}

impl SpriteAtlas {
    pub fn new(texture_atlas: TextureAtlas) -> Self {
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
        }
    }

    pub fn new_with_provider(
        provider: Box<dyn PolygonProvider>,
        texture_atlas: TextureAtlas,
    ) -> Self {
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
        SpriteAtlas {
            texture_atlas,
            polygon,
            key_map,
        }
    }

    pub fn set_atlas(&mut self, index: usize) {
        if let Some(frame) = self.texture_atlas.frames.get(index) {
            let positions = create_square_positions_from_frame(frame);
            let texcoords =
                create_square_texcoords_from_frame(self.texture_atlas.size.to_vec2(), frame);
            self.polygon
                .polygon()
                .borrow_mut()
                .core
                .update_positions_of_mesh(positions);
            self.polygon
                .polygon()
                .borrow_mut()
                .core
                .update_texcoords_of_mesh(texcoords);
            self.polygon
                .set_size(vec2(frame.rect.w as f32, frame.rect.h as f32));
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
}
