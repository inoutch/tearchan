use crate::batch::batch_command::{BatchCommandData, BatchCommandTransform, BatchObjectId};
use nalgebra_glm::{vec3_to_vec4, Mat4, TVec2, TVec3, TVec4};

pub struct BatchObject {
    pub id: BatchObjectId,
    pub data: Vec<BatchCommandData>,
    pub transforms: Vec<BatchCommandTransform>,
    pub order: i32,
}

impl BatchObject {
    pub fn copy_v2f32(&mut self, index: usize, from: &[TVec2<f32>], offset: usize) {
        match &mut self.data[index] {
            BatchCommandData::V2F32 { data } => {
                data[offset..from.len()].clone_from_slice(from);
            }
            _ => {}
        }
    }

    pub fn copy_v2u32(&mut self, index: usize, from: &[TVec2<u32>], offset: usize) {
        match &mut self.data[index] {
            BatchCommandData::V2U32 { data } => {
                data[offset..from.len()].clone_from_slice(from);
            }
            _ => {}
        }
    }

    pub fn copy_v3f32(&mut self, index: usize, from: &[TVec3<f32>], offset: usize) {
        match &mut self.data[index] {
            BatchCommandData::V3F32 { data } => {
                data[offset..from.len()].clone_from_slice(from);
            }
            _ => {}
        }
    }

    pub fn copy_v3u32(&mut self, index: usize, from: &[TVec3<u32>], offset: usize) {
        match &mut self.data[index] {
            BatchCommandData::V3U32 { data } => {
                data[offset..from.len()].clone_from_slice(from);
            }
            _ => {}
        }
    }

    pub fn copy_v4f32(&mut self, index: usize, from: &[TVec4<f32>], offset: usize) {
        match &mut self.data[index] {
            BatchCommandData::V4F32 { data } => {
                data[offset..from.len()].clone_from_slice(from);
            }
            _ => {}
        }
    }

    pub fn copy_v4u32(&mut self, index: usize, from: &[TVec4<u32>], offset: usize) {
        match &mut self.data[index] {
            BatchCommandData::V4U32 { data } => {
                data[offset..from.len()].clone_from_slice(from);
            }
            _ => {}
        }
    }

    pub fn for_each_v2f32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, f32),
    {
        match &self.data[index] {
            BatchCommandData::V2F32 { data } => {
                let mut i = 0usize;
                for datum in data {
                    callback(i, datum.x);
                    i += 1;

                    callback(i, datum.y);
                    i += 1;
                }
            }
            _ => {}
        }
    }

    pub fn for_each_v2u32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, u32),
    {
        match &self.data[index] {
            BatchCommandData::V2U32 { data } => {
                let mut i = 0usize;
                for datum in data {
                    callback(i, datum.x);
                    i += 1;

                    callback(i, datum.y);
                    i += 1;
                }
            }
            _ => {}
        }
    }

    pub fn for_each_v3f32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, f32),
    {
        match &self.data[index] {
            BatchCommandData::V3F32 { data } => {
                let mut i = 0usize;
                let m = match &self.transforms[index] {
                    BatchCommandTransform::Mat4 { m } => m.clone_owned(),
                    _ => Mat4::identity(),
                };
                for datum in data {
                    let p = &m * vec3_to_vec4(&datum);
                    callback(i, p.x);
                    i += 1;

                    callback(i, p.y);
                    i += 1;

                    callback(i, p.z);
                    i += 1;
                }
            }
            _ => {}
        }
    }

    pub fn for_each_v3u32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, u32),
    {
        match &self.data[index] {
            BatchCommandData::V3U32 { data } => {
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
            _ => {}
        }
    }

    pub fn for_each_v4f32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, f32),
    {
        match &self.data[index] {
            BatchCommandData::V4F32 { data } => {
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
            _ => {}
        }
    }

    pub fn for_each_v4u32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, u32),
    {
        match &self.data[index] {
            BatchCommandData::V4U32 { data } => {
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
            _ => {}
        }
    }
}
