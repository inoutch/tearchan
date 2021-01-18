use nalgebra_glm::{mat4, Mat4};

#[inline]
pub fn inverse_transpose(m: Mat4) -> Mat4 {
    nalgebra_glm::inverse_transpose(m)
}

pub fn create_orthographic(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    if cfg!(feature = "gl") {
        create_orthographic_for_gl(left, right, bottom, top, near, far)
    } else {
        create_orthographic_for_vulkan(left, right, bottom, top, near, far)
    }
}

fn create_orthographic_for_gl(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    // for opengl [z:-1.0 ~ 1.0]
    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);
    let tz = -(far + near) / (far - near);

    mat4(
        2.0f32 / (right - left),
        0.0f32,
        0.0f32,
        tx,
        0.0f32,
        2.0f32 / (top - bottom),
        0.0f32,
        ty,
        0.0f32,
        0.0f32,
        2.0f32 / (far - near),
        tz,
        0.0f32,
        0.0f32,
        tz,
        1.0f32,
    )
}

fn create_orthographic_for_vulkan(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    // for vulkan [z:0.0 ~ 1.0]
    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);
    let tz = -(near) / (far - near);

    mat4(
        2.0f32 / (right - left),
        0.0f32,
        0.0f32,
        tx,
        0.0f32,
        2.0f32 / (top - bottom),
        0.0f32,
        ty,
        0.0f32,
        0.0f32,
        1.0f32 / (far - near),
        tz,
        0.0f32,
        0.0f32,
        0.0f32,
        1.0f32,
    )
}

#[cfg(test)]
mod test {
    use crate::math::mat::{create_orthographic_for_gl, create_orthographic_for_vulkan};

    #[test]
    fn test_orthographic_for_vulkan() {
        let m4 = create_orthographic_for_vulkan(
            0.0f32,
            730.0f32,
            0.0f32,
            1300.0f32,
            -10000.0f32,
            10000.0f32,
        );

        assert!(
            float_cmp::approx_eq!(f32, m4[0], 0.002_739_726_f32),
            "m4[0]={}",
            m4[0]
        );
        assert!(float_cmp::approx_eq!(f32, m4[1], 0.0f32), "m4[1]={}", m4[1]);
        assert!(float_cmp::approx_eq!(f32, m4[2], 0.0f32), "m4[2]={}", m4[2]);
        assert!(float_cmp::approx_eq!(f32, m4[3], 0.0f32), "m4[3]={}", m4[3]);

        assert!(float_cmp::approx_eq!(f32, m4[4], 0.0f32), "m4[0]={}", m4[4]);
        assert!(
            float_cmp::approx_eq!(f32, m4[5], 0.001_538_461_5_f32),
            "m4[5]={}",
            m4[5]
        );
        assert!(float_cmp::approx_eq!(f32, m4[6], 0.0f32), "m4[6]={}", m4[6]);
        assert!(float_cmp::approx_eq!(f32, m4[7], 0.0f32), "m4[7]={}", m4[7]);

        assert!(float_cmp::approx_eq!(f32, m4[8], 0.0f32), "m4[8]={}", m4[8]);
        assert!(float_cmp::approx_eq!(f32, m4[9], 0.0f32), "m4[9]={}", m4[9]);
        assert!(
            float_cmp::approx_eq!(f32, m4[10], 5.0E-5f32),
            "m4[10]={}",
            m4[10]
        );
        assert!(
            float_cmp::approx_eq!(f32, m4[11], 0.0f32),
            "m4[11]={}",
            m4[11]
        );

        assert!(
            float_cmp::approx_eq!(f32, m4[12], -1.0f32),
            "m4[12]={}",
            m4[12]
        );
        assert!(
            float_cmp::approx_eq!(f32, m4[13], -1.0f32),
            "m4[13]={}",
            m4[13]
        );
        assert!(
            float_cmp::approx_eq!(f32, m4[14], 0.5f32),
            "m4[14]={}",
            m4[14]
        );
        assert!(
            float_cmp::approx_eq!(f32, m4[15], 1.0f32),
            "m4[15]={}",
            m4[15]
        );
    }

    #[test]
    fn test_orthographic_for_gl() {
        let m4 = create_orthographic_for_gl(
            0.0f32,
            730.0f32,
            0.0f32,
            1300.0f32,
            -10000.0f32,
            10000.0f32,
        );

        assert!(
            float_cmp::approx_eq!(f32, m4[0], 0.002_739_726_f32),
            "m4[0]={}",
            m4[0]
        );
        assert!(float_cmp::approx_eq!(f32, m4[1], 0.0f32), "m4[1]={}", m4[1]);
        assert!(float_cmp::approx_eq!(f32, m4[2], 0.0f32), "m4[2]={}", m4[2]);
        assert!(float_cmp::approx_eq!(f32, m4[3], 0.0f32), "m4[3]={}", m4[3]);

        assert!(float_cmp::approx_eq!(f32, m4[4], 0.0f32), "m4[0]={}", m4[4]);
        assert!(
            float_cmp::approx_eq!(f32, m4[5], 0.001_538_461_5),
            "m4[5]={}",
            m4[5]
        );
        assert!(float_cmp::approx_eq!(f32, m4[6], 0.0f32), "m4[6]={}", m4[6]);
        assert!(float_cmp::approx_eq!(f32, m4[7], 0.0f32), "m4[7]={}", m4[7]);

        assert!(float_cmp::approx_eq!(f32, m4[8], 0.0f32), "m4[8]={}", m4[8]);
        assert!(float_cmp::approx_eq!(f32, m4[9], 0.0f32), "m4[9]={}", m4[9]);
        assert!(
            float_cmp::approx_eq!(f32, m4[10], 1.0E-4f32),
            "m4[10]={}",
            m4[10]
        );
        assert!(
            float_cmp::approx_eq!(f32, m4[11], 0.0f32),
            "m4[11]={}",
            m4[11]
        );

        assert!(
            float_cmp::approx_eq!(f32, m4[12], -1.0f32),
            "m4[12]={}",
            m4[12]
        );
        assert!(
            float_cmp::approx_eq!(f32, m4[13], -1.0f32),
            "m4[13]={}",
            m4[13]
        );
        assert!(
            float_cmp::approx_eq!(f32, m4[14], 0.0f32),
            "m4[14]={}",
            m4[14]
        );
        assert!(
            float_cmp::approx_eq!(f32, m4[15], 1.0f32),
            "m4[15]={}",
            m4[15]
        );
    }
}
