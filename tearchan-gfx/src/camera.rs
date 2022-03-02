use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{vec3, Mat4, Vec2, Vec3};
use std::f32::consts::PI;
use tearchan_util::math::mat::create_orthographic;

#[derive(Clone, Debug)]
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

    pub fn billboard(&self) -> Billboard {
        Billboard {
            camera_right: vec3(
                self.view_matrix.data.0[0][0],
                self.view_matrix.data.0[1][0],
                self.view_matrix.data.0[2][0],
            ),
            camera_up: vec3(
                self.view_matrix.data.0[0][1],
                self.view_matrix.data.0[1][1],
                self.view_matrix.data.0[2][1],
            ),
            _pad1: 0.0f32,
            _pad2: 0.0f32,
        }
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Camera3D {
    base: Camera,
    pub position: Vec3,
    pub target_position: Vec3,
    pub up: Vec3,
}

impl Camera3D {
    pub fn default_with_aspect(aspect: f32) -> Self {
        Camera3D::new(aspect, 0.01f32, 10.0f32)
    }

    pub fn new(aspect: f32, near: f32, far: f32) -> Self {
        let proj_matrix = nalgebra_glm::perspective(aspect, 45.0f32 / 180.0f32 * PI, near, far);
        let view_matrix: Mat4 = nalgebra_glm::look_at(
            &vec3(0.0f32, 0.0f32, 0.0f32),
            &vec3(0.0f32, 0.0f32, 0.0f32),
            &vec3(0.0f32, 0.0f32, 0.0f32),
        );
        let base = Camera::new(proj_matrix, view_matrix);
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

    pub fn base(&self) -> &Camera {
        &self.base
    }

    pub fn combine(&self) -> &Mat4 {
        self.base.combine()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Billboard {
    pub camera_right: Vec3,
    _pad1: f32, // alignment
    pub camera_up: Vec3,
    _pad2: f32,
}

unsafe impl Zeroable for Billboard {}

unsafe impl Pod for Billboard {}
