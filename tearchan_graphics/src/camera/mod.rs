use nalgebra_glm::Mat4;

pub mod camera_2d;
pub mod camera_3d;

#[derive(Clone)]
pub struct Camera {
    pub projection_matrix: Mat4,
    pub view_matrix: Mat4,
    combine: Mat4,
}

impl Camera {
    pub fn new(projection_matrix: Mat4, view_matrix: Mat4) -> Self {
        #![allow(clippy::op_ref)]
        let combine = &projection_matrix * &view_matrix;
        Camera {
            projection_matrix,
            view_matrix,
            combine,
        }
    }

    pub fn update(&mut self) {
        let p = &self.projection_matrix;
        let v = &self.view_matrix;
        self.combine = p * v;
    }

    pub fn combine(&self) -> &Mat4 {
        &self.combine
    }
}
