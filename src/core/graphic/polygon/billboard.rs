use crate::core::graphic::polygon::polygon_2d::Polygon2DProvider;
use crate::core::graphic::polygon::sprite_atlas::SpriteAtlas;
use crate::core::graphic::polygon::{Polygon, PolygonCommon, PolygonCore, PolygonProvider};
use crate::core::graphic::texture::TextureAtlas;
use crate::extension::shared::Shared;
use crate::math::change_range::ChangeRange;
use crate::utility::buffer_interface::BufferInterface;
use nalgebra_glm::{rotate, scale, translate, vec2, Mat4, Vec3};

pub struct BillboardProvider {
    polygon_2d_provider: Polygon2DProvider,
    origin_change_range: ChangeRange,
}

impl PolygonProvider for BillboardProvider {
    fn set_position(&mut self, core: &mut PolygonCore, position: Vec3) {
        core.position = position;
        self.origin_change_range.update_all();
    }

    fn transform(&self, core: &PolygonCore) -> Mat4 {
        translate(
            &self.billboard_transform(core),
            &self.polygon_2d_provider.transform_anchor_point(),
        )
    }

    fn transform_for_child(&self, core: &PolygonCore) -> Mat4 {
        translate(
            &Mat4::identity(),
            &self.polygon_2d_provider.transform_anchor_point_for_child(),
        ) * self.transform(core)
    }
}

impl BillboardProvider {
    fn billboard_transform(&self, core: &PolygonCore) -> Mat4 {
        let current = scale(
            &rotate(
                &Mat4::identity(),
                self.rotation_radian(core),
                self.rotation_axis(core),
            ),
            self.scale(core),
        );
        if let Some(x) = core.parent() {
            return x.borrow().transform_for_child() * current;
        }
        current
    }

    fn reset_all_origin_change_range(&mut self) {
        self.origin_change_range.reset();
    }
}

pub struct Billboard {
    polygon: SpriteAtlas,
}

impl Billboard {
    pub fn new(texture_atlas: TextureAtlas) -> Self {
        let size = {
            let frame = texture_atlas.frames.first().unwrap();
            vec2(frame.rect.w as f32, frame.rect.h as f32)
        };
        let rect_size = 6;
        let provider = BillboardProvider {
            polygon_2d_provider: Polygon2DProvider::new(size),
            origin_change_range: ChangeRange::new(rect_size),
        };
        Billboard {
            polygon: SpriteAtlas::new_with_provider(Box::new(provider), texture_atlas),
        }
    }

    pub fn copy_origins_into<TBuffer: BufferInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
    ) {
        let mut polygon = self.polygon.polygon().borrow_mut();
        let position = polygon.position().to_owned();
        let provider: &mut BillboardProvider =
            polygon.provider_as_any_mut().downcast_mut().unwrap();

        let change_range = &provider.origin_change_range;
        if let Some(range) = change_range.get_range() {
            buffer.update_with_range(range.start * 3, range.end * 3);
            for i in range {
                buffer.copy(offset + i * 3, position.x);
                buffer.copy(offset + i * 3 + 1, position.y);
                buffer.copy(offset + i * 3 + 2, position.z);
            }

            provider.reset_all_origin_change_range();
        }
    }

    #[inline]
    pub fn polygon(&self) -> &Shared<Polygon> {
        &self.polygon.polygon()
    }
}
