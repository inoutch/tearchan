use crate::core::graphic::polygon::{Polygon, PolygonCore, PolygonProvider};
use crate::extension::shared::{clone_shared, make_shared, Shared};
use crate::math::mesh::Mesh;
use nalgebra_glm::{translate, vec2, vec3, Mat4, Vec2, Vec3};

pub trait Polygon2DProviderInterface {
    fn anchor_point(&self, core: &PolygonCore) -> &Vec2;
    fn size(&self, core: &PolygonCore) -> &Vec2;
    fn set_anchor_point(&mut self, core: &mut PolygonCore, anchor_point: Vec2);
    fn set_size(&mut self, core: &mut PolygonCore, size: Vec2);
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
    fn anchor_point(&self, _core: &PolygonCore) -> &Vec2 {
        &self.anchor_point
    }

    fn size(&self, _core: &PolygonCore) -> &Vec2 {
        &self.size
    }

    fn set_anchor_point(&mut self, core: &mut PolygonCore, anchor_point: Vec2) {
        if self.anchor_point != anchor_point {
            self.anchor_point = anchor_point;
            core.update_all_positions();
        }
    }

    fn set_size(&mut self, core: &mut PolygonCore, size: Vec2) {
        if self.size != size {
            self.size = size;
            core.update_all_positions();
        }
    }
}

pub trait Polygon2DInterface {
    fn set_anchor_point(&mut self, anchor_point: Vec2);
    fn set_size(&mut self, size: Vec2);
    fn size(&self) -> Vec2;
}

pub struct Polygon2D {
    polygon: Shared<Polygon>,
    provider: Shared<Polygon2DProvider>,
}

impl Polygon2D {
    pub fn new(mesh: Mesh, size: Vec2) -> Self {
        let provider = make_shared(Polygon2DProvider::new(size));
        let cloned_provider = clone_shared(&provider);
        Polygon2D {
            polygon: make_shared(Polygon::new_with_provider(provider, mesh)),
            provider: cloned_provider,
        }
    }

    pub fn polygon(&self) -> &Shared<Polygon> {
        &self.polygon
    }
}

impl Polygon2DInterface for Polygon2D {
    fn set_anchor_point(&mut self, anchor_point: Vec2) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .set_anchor_point(&mut polygon.core, anchor_point);
    }

    fn set_size(&mut self, size: Vec2) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider.borrow_mut().set_size(&mut polygon.core, size);
    }

    fn size(&self) -> Vec2 {
        let polygon = self.polygon.borrow_mut();
        self.provider.borrow_mut().size(&polygon.core).clone_owned()
    }
}
