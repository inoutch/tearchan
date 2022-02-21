use crate::batch::types::{BatchAttributeIndex, BatchTypeArray, BatchTypeTransform};
use crate::batch::v2::buffer::BatchBufferPointer;
use nalgebra_glm::{TVec2, TVec3, TVec4};

pub struct BatchObject {
    pointer: BatchBufferPointer,
    data: Vec<BatchTypeArray>,
    transforms: Vec<BatchTypeTransform>,
    order: i32,
}

impl BatchObject {
    pub fn new(
        pointer: BatchBufferPointer,
        data: Vec<BatchTypeArray>,
        transforms: Vec<BatchTypeTransform>,
        order: i32,
    ) -> Self {
        BatchObject {
            pointer,
            data,
            transforms,
            order,
        }
    }

    pub fn order(&self) -> i32 {
        self.order
    }

    pub fn pointer(&self) -> BatchBufferPointer {
        self.pointer
    }

    pub fn data(&self) -> &Vec<BatchTypeArray> {
        &self.data
    }

    pub fn set_transform(&mut self, attribute: BatchAttributeIndex, transform: BatchTypeTransform) {
        self.transforms[attribute as usize] = transform;
    }

    pub fn set_data(&mut self, attribute: BatchAttributeIndex, data: BatchTypeArray) {
        self.data[attribute as usize] = data;
    }

    pub fn set_pointer(&mut self, pointer: BatchBufferPointer) {
        self.pointer = pointer;
    }

    pub fn get_transformed_data(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<BatchTypeArray, BatchObjectError> {
        let data = self.data.get(attribute as usize).ok_or_else(|| {
            BatchObjectError::AttributeReferenceToDataOutOfRange {
                attribute,
                data_len: self.data.len(),
            }
        })?;
        if let Some(transform) = self.transforms.get(attribute as usize) {
            data.transform(transform)
                .ok_or(BatchObjectError::InvalidTransformType)
        } else {
            Ok(data.clone())
        }
    }

    pub fn get_v1f32_data(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<f32>, BatchObjectError> {
        match self.get_transformed_data(attribute)? {
            BatchTypeArray::V1F32 { data } => Ok(data),
            _ => Err(BatchObjectError::InvalidArrayType),
        }
    }

    pub fn get_v1u32_data(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<u32>, BatchObjectError> {
        match self.get_transformed_data(attribute)? {
            BatchTypeArray::V1U32 { data } => Ok(data),
            _ => Err(BatchObjectError::InvalidArrayType),
        }
    }

    pub fn get_v2f32_data(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec2<f32>>, BatchObjectError> {
        match self.get_transformed_data(attribute)? {
            BatchTypeArray::V2F32 { data } => Ok(data),
            _ => Err(BatchObjectError::InvalidArrayType),
        }
    }

    pub fn get_v2u32_data(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec2<u32>>, BatchObjectError> {
        match self.get_transformed_data(attribute)? {
            BatchTypeArray::V2U32 { data } => Ok(data),
            _ => Err(BatchObjectError::InvalidArrayType),
        }
    }

    pub fn get_v3f32_data(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec3<f32>>, BatchObjectError> {
        match self.get_transformed_data(attribute)? {
            BatchTypeArray::V3F32 { data } => Ok(data),
            _ => Err(BatchObjectError::InvalidArrayType),
        }
    }

    pub fn get_v3u32_data(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec3<u32>>, BatchObjectError> {
        match self.get_transformed_data(attribute)? {
            BatchTypeArray::V3U32 { data } => Ok(data),
            _ => Err(BatchObjectError::InvalidArrayType),
        }
    }

    pub fn get_v4f32_data(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec4<f32>>, BatchObjectError> {
        match self.get_transformed_data(attribute)? {
            BatchTypeArray::V4F32 { data } => Ok(data),
            _ => Err(BatchObjectError::InvalidArrayType),
        }
    }

    pub fn get_v4u32_data(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec4<u32>>, BatchObjectError> {
        match self.get_transformed_data(attribute)? {
            BatchTypeArray::V4U32 { data } => Ok(data),
            _ => Err(BatchObjectError::InvalidArrayType),
        }
    }
}

#[derive(Debug)]
pub enum BatchObjectError {
    AttributeReferenceToDataOutOfRange { attribute: u32, data_len: usize },
    InvalidTransformType,
    InvalidArrayType,
}
