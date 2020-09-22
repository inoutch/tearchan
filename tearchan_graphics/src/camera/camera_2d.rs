use crate::camera::Camera;
use nalgebra_glm::{vec3, Mat4, Vec2, Vec3};
use tearchan_utility::math::mat::create_orthographic;

#[derive(Clone)]
pub struct Camera2D {
    base: Camera,
    pub position: Vec3,
}

impl Camera2D {
    pub fn new(size: &Vec2) -> Self {
        let proj_matrix =
            create_orthographic(0.0f32, size.x, 0.0f32, size.y, -1000.0f32, 1000.0f32);
        let view_matrix: Mat4 = nalgebra_glm::translation(&vec3(0.0f32, 0.0f32, 0.0f32));
        Camera2D {
            base: Camera::new(proj_matrix, view_matrix),
            position: vec3(0.0f32, 0.0f32, 0.0f32),
        }
    }

    pub fn update(&mut self) {
        self.base.view_matrix =
            nalgebra_glm::translation(&vec3(-self.position.x, -self.position.y, -self.position.z));
        self.base.update();
    }

    pub fn base(&self) -> &Camera {
        &self.base
    }

    pub fn combine(&self) -> &Mat4 {
        self.base.combine()
    }
}
