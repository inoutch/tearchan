use crate::batch::batch_command::{BatchCommandData, BatchCommandTransform, BatchObjectId};
use nalgebra_glm::{vec4, Mat4, TVec2, TVec3, TVec4};

pub struct BatchObject {
    pub id: BatchObjectId,
    pub data: Vec<BatchCommandData>,
    pub transforms: Vec<BatchCommandTransform>,
    pub order: i32,
}

impl BatchObject {
    pub fn copy_v2f32(&mut self, index: usize, from: &[TVec2<f32>], offset: usize) {
        if let BatchCommandData::V2F32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v2u32(&mut self, index: usize, from: &[TVec2<u32>], offset: usize) {
        if let BatchCommandData::V2U32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v3f32(&mut self, index: usize, from: &[TVec3<f32>], offset: usize) {
        if let BatchCommandData::V3F32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v3u32(&mut self, index: usize, from: &[TVec3<u32>], offset: usize) {
        if let BatchCommandData::V3U32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v4f32(&mut self, index: usize, from: &[TVec4<f32>], offset: usize) {
        if let BatchCommandData::V4F32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v4u32(&mut self, index: usize, from: &[TVec4<u32>], offset: usize) {
        if let BatchCommandData::V4U32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn for_each_v1f32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, f32),
    {
        if let BatchCommandData::V1F32 { data } = &self.data[index] {
            for (i, datum) in data.iter().enumerate() {
                callback(i, *datum);
            }
        }
    }

    pub fn for_each_v1u32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, u32),
    {
        if let BatchCommandData::V1U32 { data } = &self.data[index] {
            for (i, datum) in data.iter().enumerate() {
                callback(i, *datum);
            }
        }
    }

    pub fn for_each_v2f32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, f32),
    {
        if let BatchCommandData::V2F32 { data } = &self.data[index] {
            let mut i = 0usize;
            for datum in data {
                callback(i, datum.x);
                i += 1;

                callback(i, datum.y);
                i += 1;
            }
        }
    }

    pub fn for_each_v2u32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, u32),
    {
        if let BatchCommandData::V2U32 { data } = &self.data[index] {
            let mut i = 0usize;
            for datum in data {
                callback(i, datum.x);
                i += 1;

                callback(i, datum.y);
                i += 1;
            }
        }
    }

    pub fn for_each_v3f32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, f32),
    {
        if let BatchCommandData::V3F32 { data } = &self.data[index] {
            let mut i = 0usize;
            let m = match &self.transforms[index] {
                BatchCommandTransform::Mat4 { m } => m.clone_owned(),
                _ => Mat4::identity(),
            };
            for datum in data {
                #[allow(clippy::op_ref)]
                let p = &m * vec4(datum.x, datum.y, datum.z, 1.0f32);
                callback(i, p.x);
                i += 1;

                callback(i, p.y);
                i += 1;

                callback(i, p.z);
                i += 1;
            }
        }
    }

    pub fn for_each_v3u32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, u32),
    {
        if let BatchCommandData::V3U32 { data } = &self.data[index] {
            let mut i = 0usize;
            for datum in data {
                callback(i, datum.x);
                i += 1;

                callback(i, datum.y);
                i += 1;

                callback(i, datum.z);
                i += 1;
            }
        }
    }

    pub fn for_each_v4f32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, f32),
    {
        if let BatchCommandData::V4F32 { data } = &self.data[index] {
            let mut i = 0usize;
            for datum in data {
                callback(i, datum.x);
                i += 1;

                callback(i, datum.y);
                i += 1;

                callback(i, datum.z);
                i += 1;

                callback(i, datum.w);
                i += 1;
            }
        }
    }

    pub fn for_each_v4u32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, u32),
    {
        if let BatchCommandData::V4U32 { data } = &self.data[index] {
            let mut i = 0usize;
            for datum in data {
                callback(i, datum.x);
                i += 1;

                callback(i, datum.y);
                i += 1;

                callback(i, datum.z);
                i += 1;

                callback(i, datum.w);
                i += 1;
            }
        }
    }
}
