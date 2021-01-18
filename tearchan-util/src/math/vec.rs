use nalgebra_glm::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

#[inline]
pub fn vec4_white() -> Vec4 {
    vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32)
}

#[inline]
pub fn vec2_zero() -> Vec2 {
    vec2(0.0f32, 0.0f32)
}

#[inline]
pub fn vec3_zero() -> Vec3 {
    vec3(0.0f32, 0.0f32, 0.0f32)
}

#[inline]
pub fn vec2_one() -> Vec2 {
    vec2(1.0f32, 1.0f32)
}

#[inline]
pub fn vec3_one() -> Vec3 {
    vec3(1.0f32, 1.0f32, 1.0f32)
}
