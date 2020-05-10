use crate::core::graphic::camera::CameraBase;
use nalgebra_glm::{mat4, vec3, Mat4, Vec3};
use std::f32::consts::PI;

pub struct Camera3D {
    base: CameraBase,
    pub position: Vec3,
    pub target_position: Vec3,
    pub up: Vec3,
}

impl Camera3D {
    pub fn new(aspect: f32) -> Self {
        /*let proj_matrix = mat4(
            1.7808219f32,
            0.0f32,
            0.0f32,
            0.0f32,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            -1.0101011,
            -0.10101011,
            0.0,
            0.0,
            -1.0,
            0.0,
        );*/
        let proj_matrix =
            nalgebra_glm::perspective(aspect, 45.0f32 / 180.0f32 * PI, 0.1f32, 10.0f32);
        println!("proj={:?}", proj_matrix);
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

    pub fn borrow_base(&self) -> &CameraBase {
        &self.base
    }

    pub fn borrow_combine(&self) -> &Mat4 {
        self.base.borrow_combine()
    }
}
