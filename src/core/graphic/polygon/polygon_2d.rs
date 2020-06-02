use crate::core::graphic::polygon::{Polygon, PolygonCore, PolygonProvider};
use crate::extension::shared::{Shared, make_shared};
use crate::math::mesh::Mesh;
use nalgebra_glm::{translate, vec2, vec3, Mat4, Vec2};

pub struct Polygon2DProvider {
    anchor_point: Vec2,
    size: Vec2,
}

impl PolygonProvider for Polygon2DProvider {
    fn transform(&self, core: &PolygonCore) -> Mat4 {
        let p = vec3(
            -self.size.x * self.anchor_point.x,
            -self.size.y * self.anchor_point.y,
            0.0f32,
        );
        translate(&core.transform(self), &p)
    }

    fn transform_for_child(&self, core: &PolygonCore) -> Mat4 {
        let p = vec3(self.size.x * 0.5f32, self.size.y * 0.5f32, 0.0f32);
        translate(&Mat4::identity(), &p) * self.transform(core)
    }
}

pub struct Polygon2D {
    polygon: Shared<Polygon>,
}

impl Polygon2D {
    pub fn new(mesh: Mesh, size: Vec2) -> Self {
        let provider = Box::new(Polygon2DProvider {
            anchor_point: vec2(0.5f32, 0.5f32),
            size,
        });
        Polygon2D {
            polygon: make_shared(Polygon::new_with_provider(provider, mesh)),
        }
    }

    pub fn set_anchor_point(&mut self, anchor_point: Vec2) {
        let is_changed = {
            let mut polygon = self.polygon.borrow_mut();
            let provider: &mut Polygon2DProvider =
                polygon.provider_as_any_mut().downcast_mut().unwrap();
            provider.anchor_point = anchor_point;
            provider.anchor_point == anchor_point
        };

        if is_changed {
            self.polygon.borrow_mut().core.update_all_positions();
        }
    }

    pub fn polygon(&mut self) -> &Shared<Polygon> {
        &self.polygon
    }
}
