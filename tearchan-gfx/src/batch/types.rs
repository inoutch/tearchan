use nalgebra_glm::{
    vec2, vec3, vec3_to_vec2, vec4, vec4_to_vec3, Mat2, Mat3, Mat4, TMat2, TMat3, TMat4, TVec2,
    TVec3, TVec4, Vec2, Vec3, Vec4,
};
use std::option::Option::Some;

#[derive(Clone, Debug)]
pub enum BatchTypeValue {
    Vec3 { v: Vec3 },
    Vec4 { v: Vec4 },
    U32 { v: u32 },
}

#[derive(Clone, Debug)]
pub enum BatchTypeArray {
    V1F32 { data: Vec<f32> },
    V1U32 { data: Vec<u32> },
    V2F32 { data: Vec<TVec2<f32>> },
    V2U32 { data: Vec<TVec2<u32>> },
    V3F32 { data: Vec<TVec3<f32>> },
    V3U32 { data: Vec<TVec3<u32>> },
    V4F32 { data: Vec<TVec4<f32>> },
    V4U32 { data: Vec<TVec4<u32>> },
}

impl BatchTypeArray {
    pub fn len(&self) -> usize {
        match self {
            Self::V1F32 { data } => data.len(),
            Self::V1U32 { data } => data.len(),
            Self::V2F32 { data } => data.len(),
            Self::V2U32 { data } => data.len(),
            Self::V3F32 { data } => data.len(),
            Self::V3U32 { data } => data.len(),
            Self::V4F32 { data } => data.len(),
            Self::V4U32 { data } => data.len(),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            BatchTypeArray::V1F32 { .. } => "v1f32",
            BatchTypeArray::V1U32 { .. } => "v1u32",
            BatchTypeArray::V2F32 { .. } => "v2f32",
            BatchTypeArray::V2U32 { .. } => "v2u32",
            BatchTypeArray::V3F32 { .. } => "v3f32",
            BatchTypeArray::V3U32 { .. } => "v3u32",
            BatchTypeArray::V4F32 { .. } => "v4f32",
            BatchTypeArray::V4U32 { .. } => "v4u32",
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_v1f32(&self) -> Option<&Vec<f32>> {
        match self {
            BatchTypeArray::V1F32 { data } => Some(data),
            _ => None,
        }
    }

    pub fn get_v1u32(&self) -> Option<&Vec<u32>> {
        match self {
            BatchTypeArray::V1U32 { data } => Some(data),
            _ => None,
        }
    }

    pub fn get_v2f32(&self) -> Option<&Vec<Vec2>> {
        match self {
            BatchTypeArray::V2F32 { data } => Some(data),
            _ => None,
        }
    }

    pub fn get_v3f32(&self) -> Option<&Vec<Vec3>> {
        match self {
            BatchTypeArray::V3F32 { data } => Some(data),
            _ => None,
        }
    }

    pub fn get_v4f32(&self) -> Option<&Vec<Vec4>> {
        match self {
            BatchTypeArray::V4F32 { data } => Some(data),
            _ => None,
        }
    }

    pub fn transform(&self, transform: &BatchTypeTransform) -> Result<Self, BatchTypeArrayError> {
        match transform {
            BatchTypeTransform::Mat2U32 { m } => {
                if let BatchTypeArray::V1U32 { data } = self {
                    let transformed = data
                        .iter()
                        .map(|x| {
                            let v = m * vec2(*x, 1u32);
                            v.x
                        })
                        .collect();
                    return Ok(BatchTypeArray::V1U32 { data: transformed });
                }
                return Err(BatchTypeArrayError::InvalidTransformType {
                    expect: "v1u32",
                    actual: self.label(),
                });
            }
            BatchTypeTransform::Mat2F32 { m } => {
                if let BatchTypeArray::V1F32 { data } = self {
                    let transformed = data
                        .iter()
                        .map(|x| {
                            let v = m * vec2(*x, 1f32);
                            v.x
                        })
                        .collect();
                    return Ok(BatchTypeArray::V1F32 { data: transformed });
                }
                return Err(BatchTypeArrayError::InvalidTransformType {
                    expect: "v1f32",
                    actual: self.label(),
                });
            }
            BatchTypeTransform::Mat3U32 { m } => {
                if let BatchTypeArray::V2U32 { data } = self {
                    let transformed = data
                        .iter()
                        .map(|x| {
                            let v = m * vec3(x.x, x.y, 1u32);
                            vec3_to_vec2(&v)
                        })
                        .collect();
                    return Ok(BatchTypeArray::V2U32 { data: transformed });
                }
                return Err(BatchTypeArrayError::InvalidTransformType {
                    expect: "v2u32",
                    actual: self.label(),
                });
            }
            BatchTypeTransform::Mat3F32 { m } => {
                if let BatchTypeArray::V2F32 { data } = self {
                    let transformed = data
                        .iter()
                        .map(|x| {
                            let v = m * vec3(x.x, x.y, 1f32);
                            vec3_to_vec2(&v)
                        })
                        .collect();
                    return Ok(BatchTypeArray::V2F32 { data: transformed });
                }
                return Err(BatchTypeArrayError::InvalidTransformType {
                    expect: "v2f32",
                    actual: self.label(),
                });
            }
            BatchTypeTransform::Mat4U32 { m } => {
                if let BatchTypeArray::V3U32 { data } = self {
                    let transformed = data
                        .iter()
                        .map(|x| {
                            let v = m * vec4(x.x, x.y, x.z, 1u32);
                            vec4_to_vec3(&v)
                        })
                        .collect();
                    return Ok(BatchTypeArray::V3U32 { data: transformed });
                }
                return Err(BatchTypeArrayError::InvalidTransformType {
                    expect: "v3u32",
                    actual: self.label(),
                });
            }
            BatchTypeTransform::Mat4F32 { m } => {
                if let BatchTypeArray::V3F32 { data } = self {
                    let transformed = data
                        .iter()
                        .map(|x| {
                            let v = m * vec4(x.x, x.y, x.z, 1.0f32);
                            vec4_to_vec3(&v)
                        })
                        .collect();
                    return Ok(BatchTypeArray::V3F32 { data: transformed });
                }
                return Err(BatchTypeArrayError::InvalidTransformType {
                    expect: "v3f32",
                    actual: self.label(),
                });
            }
            BatchTypeTransform::None => {}
        }
        Ok(self.clone())
    }
}

#[derive(Debug)]
pub enum BatchTypeArrayError {
    InvalidTransformType {
        expect: &'static str,
        actual: &'static str,
    },
}

#[derive(Clone, Debug)]
pub enum BatchTypeTransform {
    Mat2U32 { m: TMat2<u32> },
    Mat2F32 { m: Mat2 },
    Mat3U32 { m: TMat3<u32> },
    Mat3F32 { m: Mat3 },
    Mat4U32 { m: TMat4<u32> },
    Mat4F32 { m: Mat4 },
    None,
}

impl BatchTypeTransform {
    pub fn label(&self) -> &'static str {
        match self {
            BatchTypeTransform::Mat2U32 { .. } => "mat2u32",
            BatchTypeTransform::Mat2F32 { .. } => "mat2f32",
            BatchTypeTransform::Mat3U32 { .. } => "mat3u32",
            BatchTypeTransform::Mat3F32 { .. } => "mat3f32",
            BatchTypeTransform::Mat4U32 { .. } => "mat4u32",
            BatchTypeTransform::Mat4F32 { .. } => "mat4f32",
            BatchTypeTransform::None => "none",
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, BatchTypeTransform::None)
    }
}

pub type BatchAttributeIndex = u32;
