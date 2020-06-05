use crate::core::graphic::camera::CameraBase;
use nalgebra_glm::{vec3, Mat4, Vec3};
use std::f32::consts::PI;

pub struct Camera3D {
    base: CameraBase,
    pub position: Vec3,
    pub target_position: Vec3,
    pub up: Vec3,
}

impl Camera3D {
    pub fn default_with_aspect(aspect: f32) -> Self {
        Camera3D::new(aspect, 0.1f32, 10.0f32)
    }

    pub fn new(aspect: f32, near: f32, far: f32) -> Self {
        let proj_matrix = nalgebra_glm::perspective(aspect, 45.0f32 / 180.0f32 * PI, near, far);
        let view_matrix: Mat4 = nalgebra_glm::look_at(
            &vec3(0.0f32, 0.0f32, 0.0f32),
            &vec3(0.0f32, 0.0f32, 0.0f32),
            &vec3(0.0f32, 0.0f32, 0.0f32),
        );
        let base = CameraBase::new(proj_matrix, view_matrix);
        Camera3D {
            base,
            position: vec3(0.0f32, 0.0f32, 0.0f32),
            target_position: vec3(0.0f32, 0.0f32, 0.0f32),
            up: vec3(0.0f32, 0.0f32, 0.0f32),
        }
    }

    pub fn update(&mut self) {
        self.base.view_matrix =
            nalgebra_glm::look_at(&self.position, &self.target_position, &self.up);
        self.base.update();
    }

    pub fn base(&self) -> &CameraBase {
        &self.base
    }

    pub fn combine(&self) -> &Mat4 {
        self.base.combine()
    }
}
