use nalgebra_glm::Mat4;

#[inline]
pub fn inverse_transpose(m: Mat4) -> Mat4 {
    nalgebra_glm::inverse_transpose(m)
}
