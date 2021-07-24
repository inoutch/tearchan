use crate::batch::types::{BatchAttributeIndex, BatchTypeArray, BatchTypeTransform};
use crate::batch::BatchObjectId;
use nalgebra_glm::{vec4, Mat4, TVec2, TVec3, TVec4};
use std::collections::HashMap;

pub struct BatchObject {
    id: BatchObjectId,
    data: Vec<BatchTypeArray>,
    data_cache: HashMap<BatchAttributeIndex, BatchTypeArray>,
    transforms: Vec<BatchTypeTransform>,
    order: i32,
}

impl BatchObject {
    pub fn new(
        id: BatchObjectId,
        data: Vec<BatchTypeArray>,
        transforms: Vec<BatchTypeTransform>,
        order: i32,
    ) -> Self {
        BatchObject {
            id,
            data,
            data_cache: HashMap::new(),
            transforms,
            order,
        }
    }

    pub fn id(&self) -> BatchObjectId {
        self.id
    }

    pub fn order(&self) -> i32 {
        self.order
    }

    pub fn set_transform(&mut self, attribute: BatchAttributeIndex, transform: BatchTypeTransform) {
        self.transforms[attribute as usize] = transform;
    }

    pub fn set_data(&mut self, attribute: BatchAttributeIndex, data: BatchTypeArray) {
        self.data[attribute as usize] = data;
    }

    pub fn copy_v2f32(&mut self, index: usize, from: &[TVec2<f32>], offset: usize) {
        if let BatchTypeArray::V2F32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v2u32(&mut self, index: usize, from: &[TVec2<u32>], offset: usize) {
        if let BatchTypeArray::V2U32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v3f32(&mut self, index: usize, from: &[TVec3<f32>], offset: usize) {
        if let BatchTypeArray::V3F32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v3u32(&mut self, index: usize, from: &[TVec3<u32>], offset: usize) {
        if let BatchTypeArray::V3U32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v4f32(&mut self, index: usize, from: &[TVec4<f32>], offset: usize) {
        if let BatchTypeArray::V4F32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn copy_v4u32(&mut self, index: usize, from: &[TVec4<u32>], offset: usize) {
        if let BatchTypeArray::V4U32 { data } = &mut self.data[index] {
            data[offset..from.len()].clone_from_slice(from);
        }
    }

    pub fn for_each_v1f32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, f32),
    {
        if let BatchTypeArray::V1F32 { data } = &self.data[index] {
            for (i, datum) in data.iter().enumerate() {
                callback(i, *datum);
            }
        }
    }

    pub fn for_each_v1u32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, u32),
    {
        if let BatchTypeArray::V1U32 { data } = &self.data[index] {
            for (i, datum) in data.iter().enumerate() {
                callback(i, *datum);
            }
        }
    }

    pub fn for_each_v2f32<F>(&self, index: usize, mut callback: F)
    where
        F: FnMut(usize, f32),
    {
        if let BatchTypeArray::V2F32 { data } = &self.data[index] {
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
        if let BatchTypeArray::V2U32 { data } = &self.data[index] {
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
        if let BatchTypeArray::V3F32 { data } = &self.data[index] {
            let mut i = 0usize;
            let m = match &self.transforms[index] {
                BatchTypeTransform::Mat4F32 { m } => m.clone_owned(),
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
        if let BatchTypeArray::V3U32 { data } = &self.data[index] {
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
        if let BatchTypeArray::V4F32 { data } = &self.data[index] {
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
        if let BatchTypeArray::V4U32 { data } = &self.data[index] {
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

    pub fn get_v1f32_data(&mut self, attribute: BatchAttributeIndex) -> &Vec<f32> {
        if let Some(data) =
            self.data[attribute as usize].transform(&self.transforms[attribute as usize])
        {
            self.data_cache.insert(attribute, data);
            if let BatchTypeArray::V1F32 { data } = &self.data_cache.get(&attribute).unwrap() {
                return data;
            }
        }
        if let BatchTypeArray::V1F32 { data } = &self.data[attribute as usize] {
            return data;
        }
        panic!("Invalid type as v1f32");
    }

    pub fn get_v1u32_data(&mut self, attribute: BatchAttributeIndex) -> &Vec<u32> {
        if let Some(data) =
            self.data[attribute as usize].transform(&self.transforms[attribute as usize])
        {
            self.data_cache.insert(attribute, data);
            if let BatchTypeArray::V1U32 { data } = &self.data_cache.get(&attribute).unwrap() {
                return data;
            }
        }
        if let BatchTypeArray::V1U32 { data } = &self.data[attribute as usize] {
            return data;
        }
        panic!("Invalid type as v1u32");
    }

    pub fn get_v2f32_data(&mut self, attribute: BatchAttributeIndex) -> &Vec<TVec2<f32>> {
        if let Some(data) =
            self.data[attribute as usize].transform(&self.transforms[attribute as usize])
        {
            self.data_cache.insert(attribute, data);
            if let BatchTypeArray::V2F32 { data } = &self.data_cache.get(&attribute).unwrap() {
                return data;
            }
        }
        if let BatchTypeArray::V2F32 { data } = &self.data[attribute as usize] {
            return data;
        }
        panic!("Invalid type as v2f32");
    }

    pub fn get_v2u32_data(&mut self, attribute: BatchAttributeIndex) -> &Vec<TVec2<u32>> {
        if let Some(data) =
            self.data[attribute as usize].transform(&self.transforms[attribute as usize])
        {
            self.data_cache.insert(attribute, data);
            if let BatchTypeArray::V2U32 { data } = &self.data_cache.get(&attribute).unwrap() {
                return data;
            }
        }
        if let BatchTypeArray::V2U32 { data } = &self.data[attribute as usize] {
            return data;
        }
        panic!("Invalid type as v2u32");
    }

    pub fn get_v3f32_data(&mut self, attribute: BatchAttributeIndex) -> &Vec<TVec3<f32>> {
        if let Some(data) =
            self.data[attribute as usize].transform(&self.transforms[attribute as usize])
        {
            self.data_cache.insert(attribute, data);
            if let BatchTypeArray::V3F32 { data } = &self.data_cache.get(&attribute).unwrap() {
                return data;
            }
        }
        if let BatchTypeArray::V3F32 { data } = &self.data[attribute as usize] {
            return data;
        }
        panic!("Invalid type as v3f32");
    }

    pub fn get_v3u32_data(&mut self, attribute: BatchAttributeIndex) -> &Vec<TVec3<u32>> {
        if let Some(data) =
            self.data[attribute as usize].transform(&self.transforms[attribute as usize])
        {
            self.data_cache.insert(attribute, data);
            if let BatchTypeArray::V3U32 { data } = &self.data_cache.get(&attribute).unwrap() {
                return data;
            }
        }
        if let BatchTypeArray::V3U32 { data } = &self.data[attribute as usize] {
            return data;
        }
        panic!("Invalid type as v3u32");
    }

    pub fn get_v4f32_data(&self, attribute: BatchAttributeIndex) -> &Vec<TVec4<f32>> {
        if let BatchTypeArray::V4F32 { data } = &self.data[attribute as usize] {
            return data;
        }
        panic!("Invalid type as v4f32");
    }

    pub fn get_v4u32_data(&self, attribute: BatchAttributeIndex) -> &Vec<TVec4<u32>> {
        if let BatchTypeArray::V4U32 { data } = &self.data[attribute as usize] {
            return data;
        }
        panic!("Invalid type as v4u32");
    }
}

pub enum BatchObjectCommand {
    Add {
        id: BatchObjectId,
        data: Vec<BatchTypeArray>,
        order: Option<i32>,
    },
    Remove {
        id: BatchObjectId,
    },
    Transform {
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        transform: BatchTypeTransform,
    },
    Replace {
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        data: BatchTypeArray,
    },
    Refresh {
        attribute: BatchAttributeIndex,
    },
}
