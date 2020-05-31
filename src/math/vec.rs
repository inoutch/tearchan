use nalgebra_glm::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

pub fn make_vec2_zero() -> Vec2 {
    vec2(0.0f32, 0.0f32)
}

pub fn make_vec3_zero() -> Vec3 {
    vec3(0.0f32, 0.0f32, 0.0f32)
}

pub fn make_vec4_white() -> Vec4 {
    vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32)
}

pub fn make_vec3_fill(value: f32) -> Vec3 {
    vec3(value, value, value)
}
