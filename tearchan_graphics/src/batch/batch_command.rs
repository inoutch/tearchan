use nalgebra_glm::{Mat4, TVec2, TVec3, TVec4, Vec3, Vec4};

pub type BatchObjectId = u64;
pub const BATCH_ID_EMPTY: u64 = std::u64::MAX;

pub enum BatchCommandValue {
    Vec3 { v: Vec3 },
    Vec4 { v: Vec4 },
    U32 { v: u32 },
}

pub enum BatchCommandData {
    V2F32 { data: Vec<TVec2<f32>> },
    V2U32 { data: Vec<TVec2<u32>> },
    V3F32 { data: Vec<TVec3<f32>> },
    V3U32 { data: Vec<TVec3<u32>> },
    V4F32 { data: Vec<TVec4<f32>> },
    V4U32 { data: Vec<TVec4<u32>> },
}

impl BatchCommandData {
    pub fn len(&self) -> usize {
        match self {
            BatchCommandData::V2F32 { data } => data.len(),
            BatchCommandData::V2U32 { data } => data.len(),
            BatchCommandData::V3F32 { data } => data.len(),
            BatchCommandData::V3U32 { data } => data.len(),
            BatchCommandData::V4F32 { data } => data.len(),
            BatchCommandData::V4U32 { data } => data.len(),
        }
    }
}

#[derive(Clone)]
pub enum BatchCommandTransform {
    Mat4 { m: Mat4 },
    None,
}

pub enum BatchCommand {
    Add {
        id: BatchObjectId,
        data: Vec<BatchCommandData>,
        order: Option<i32>,
    },
    Remove {
        id: BatchObjectId,
    },
    Transform {
        id: BatchObjectId,
        attribute: u32,
        transform: BatchCommandTransform,
    },
    Replace {
        id: BatchObjectId,
        attribute: u32,
        data: BatchCommandData,
    },
    CopyForEach {
        id: BatchObjectId,
        value: BatchCommandValue,
    },
}
