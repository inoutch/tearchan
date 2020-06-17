use crate::core::graphic::polygon::{Polygon, PolygonCore, PolygonProvider};
use crate::extension::shared::{make_shared, Shared};
use crate::math::mesh::Mesh;
use nalgebra_glm::{translate, vec2, vec3, Mat4, Vec2, Vec3};
use std::any::Any;

pub struct Polygon2DProvider {
    anchor_point: Vec2,
    size: Vec2,
}

impl Polygon2DProvider {
    pub fn transform_anchor_point(&self) -> Vec3 {
        vec3(
            -self.size.x * self.anchor_point.x,
            -self.size.y * self.anchor_point.y,
            0.0f32,
        )
    }

    pub fn transform_anchor_point_for_child(&self) -> Vec3 {
        vec3(self.size.x * 0.5f32, self.size.y * 0.5f32, 0.0f32)
    }
}

impl PolygonProvider for Polygon2DProvider {
    fn transform(&self, core: &PolygonCore) -> Mat4 {
        translate(&core.transform(self), &self.transform_anchor_point())
    }

    fn transform_for_child(&self, core: &PolygonCore) -> Mat4 {
        translate(&Mat4::identity(), &self.transform_anchor_point_for_child())
            * self.transform(core)
    }

    fn as_any_provider_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Polygon2DProvider {
    pub fn new(size: Vec2) -> Self {
        Polygon2DProvider {
            anchor_point: vec2(0.5f32, 0.5f32),
            size,
        }
    }
}

pub struct Polygon2D {
    polygon: Shared<Polygon>,
}

impl Polygon2D {
    pub fn new(mesh: Mesh, size: Vec2) -> Self {
        let provider = Box::new(Polygon2DProvider::new(size));
        Polygon2D {
            polygon: make_shared(Polygon::new_with_provider(provider, mesh)),
        }
    }

    pub fn new_with_provider(provider: Box<dyn PolygonProvider>, mesh: Mesh) -> Self {
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

    pub fn set_size(&mut self, size: Vec2) {
        let is_changed = {
            let mut polygon = self.polygon.borrow_mut();
            let provider: &mut Polygon2DProvider =
                polygon.provider_as_any_mut().downcast_mut().unwrap();
            provider.size = size;
            provider.size == size
        };

        if is_changed {
            self.polygon.borrow_mut().core.update_all_positions();
        }
    }

    pub fn polygon(&self) -> &Shared<Polygon> {
        &self.polygon
    }
}
