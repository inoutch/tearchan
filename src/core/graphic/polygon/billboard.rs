use crate::core::graphic::batch::batch_change_manager::BatchChangeNotifier;
use crate::core::graphic::polygon::polygon_2d::{Polygon2DProvider, Polygon2DProviderInterface};
use crate::core::graphic::polygon::sprite_atlas::{SpriteAtlas, SpriteAtlasCommon};
use crate::core::graphic::polygon::{Polygon, PolygonCommon, PolygonCore, PolygonProvider};
use crate::core::graphic::texture::TextureAtlas;
use crate::extension::shared::Shared;
use crate::math::change_range::ChangeRange;
use crate::utility::buffer_interface::BufferInterface;
use crate::utility::change_notifier::{ChangeNotifier, ChangeNotifierObject};
use nalgebra_glm::{rotate, scale, translate, vec2, Mat4, Vec2, Vec3};
use std::any::Any;

pub struct BillboardProvider {
    polygon_2d_provider: Polygon2DProvider,
    origin_change_range: ChangeRange,
    notifier: Option<BatchChangeNotifier<Billboard>>,
}

impl PolygonProvider for BillboardProvider {
    fn set_position(&mut self, core: &mut PolygonCore, position: Vec3) -> bool {
        if core.position != position {
            core.position = position;
            self.origin_change_range.update_all();
            self.request_change(core);
            true
        } else {
            false
        }
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

    fn as_any_provider_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn request_change(&mut self, _core: &mut PolygonCore) {
        if let Some(notifier) = &mut self.notifier {
            notifier.request_change();
        }
    }
}

impl Polygon2DProviderInterface for BillboardProvider {
    fn anchor_point(&self) -> &Vec2 {
        self.polygon_2d_provider.anchor_point()
    }

    fn size(&self) -> &Vec2 {
        self.polygon_2d_provider.size()
    }

    fn set_anchor_point(&mut self, anchor_point: Vec2) {
        self.polygon_2d_provider.set_anchor_point(anchor_point)
    }

    fn set_size(&mut self, size: Vec2) {
        self.polygon_2d_provider.set_size(size)
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
    polygon: SpriteAtlasCommon<BillboardProvider>,
}

impl Billboard {
    pub fn new(texture_atlas: TextureAtlas) -> Self {
        let size = {
            let frame = texture_atlas.frames.first().unwrap();
            vec2(frame.source.w as f32, frame.source.h as f32)
        };
        let rect_size = 6;
        let provider = BillboardProvider {
            polygon_2d_provider: Polygon2DProvider::new(size),
            origin_change_range: ChangeRange::new(rect_size),
            notifier: None,
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
        let position = { self.polygon.polygon().borrow().position().to_owned() };
        let mut polygon = self.polygon.polygon().borrow_mut();
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

    #[inline]
    pub fn sprite_atlas(&self) -> &SpriteAtlasCommon<BillboardProvider> {
        &self.polygon
    }

    #[inline]
    pub fn sprite_atlas_mut(&mut self) -> &mut SpriteAtlasCommon<BillboardProvider> {
        &mut self.polygon
    }

    pub fn set_anchor_point(&mut self, anchor_point: Vec2) {
        let is_changed = {
            let mut polygon = self.polygon.polygon2d_mut().polygon().borrow_mut();
            let provider: &mut BillboardProvider =
                polygon.provider_as_any_mut().downcast_mut().unwrap();
            if provider.polygon_2d_provider.anchor_point == anchor_point {
                false
            } else {
                provider.polygon_2d_provider.anchor_point = anchor_point;
                true
            }
        };

        if is_changed {
            self.polygon
                .polygon2d_mut()
                .polygon()
                .borrow_mut()
                .core
                .update_all_positions();
        }
    }
}

impl ChangeNotifierObject<BatchChangeNotifier<Billboard>> for Billboard {
    fn set_change_notifier(&mut self, notifier: BatchChangeNotifier<Billboard>) {
        let mut polygon = self.polygon.polygon2d_mut().polygon().borrow_mut();
        let provider: &mut BillboardProvider =
            polygon.provider_as_any_mut().downcast_mut().unwrap();
        let mut n = notifier;
        n.request_change();
        provider.notifier = Some(n);
    }
}
