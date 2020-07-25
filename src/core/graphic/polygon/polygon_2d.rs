use crate::core::graphic::polygon::{Polygon, PolygonCore, PolygonProvider};
use crate::extension::shared::{make_shared, Shared};
use crate::math::mesh::Mesh;
use nalgebra_glm::{translate, vec2, vec3, Mat4, Vec2, Vec3};
use std::any::Any;

pub trait Polygon2DProviderInterface {
    fn anchor_point(&self) -> &Vec2;
    fn size(&self) -> &Vec2;
    fn set_anchor_point(&mut self, anchor_point: Vec2);
    fn set_size(&mut self, size: Vec2);
}

pub struct Polygon2DProvider {
    pub anchor_point: Vec2,
    pub size: Vec2,
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

impl Polygon2DProviderInterface for Polygon2DProvider {
    fn anchor_point(&self) -> &Vec2 {
        &self.anchor_point
    }

    fn size(&self) -> &Vec2 {
        &self.size
    }

    fn set_anchor_point(&mut self, anchor_point: Vec2) {
        self.anchor_point = anchor_point;
    }

    fn set_size(&mut self, size: Vec2) {
        self.size = size;
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

    pub fn set_anchor_point<T: 'static + Polygon2DProviderInterface>(
        &mut self,
        anchor_point: Vec2,
    ) {
        let is_changed = {
            let mut polygon = self.polygon.borrow_mut();
            let provider: &mut T = polygon.provider_as_any_mut().downcast_mut().unwrap();
            if provider.anchor_point() == &anchor_point {
                false
            } else {
                provider.set_anchor_point(anchor_point);
                true
            }
        };

        if is_changed {
            self.polygon.borrow_mut().core.update_all_positions();
        }
    }

    pub fn set_size<T: 'static + Polygon2DProviderInterface>(&mut self, size: Vec2) {
        let is_changed = {
            let mut polygon = self.polygon.borrow_mut();
            let provider: &mut T = polygon.provider_as_any_mut().downcast_mut().unwrap();
            if provider.size() == &size {
                false
            } else {
                provider.set_size(size);
                false
            }
        };

        if is_changed {
            self.polygon.borrow_mut().core.update_all_positions();
        }
    }

    pub fn size<T: 'static + Polygon2DProviderInterface>(&self) -> Vec2 {
        let mut polygon = self.polygon.borrow_mut();
        let provider: &T = polygon.provider_as_any_mut().downcast_ref().unwrap();
        provider.size().clone_owned()
    }

    pub fn polygon(&self) -> &Shared<Polygon> {
        &self.polygon
    }
}

impl Polygon {
    pub fn polygon_provider_2d<T: 'static + Polygon2DProviderInterface>(&self) -> &T {
        self.provider_as_any().downcast_ref().unwrap()
    }
}
