use crate::batch::types::{
    BatchAttributeIndex, BatchTypeArray, BatchTypeArrayError, BatchTypeTransform,
};
use crate::batch::v2::buffer::BatchBufferPointer;
use nalgebra_glm::{TVec2, TVec3, TVec4};

pub struct BatchObject {
    index_pointer: BatchBufferPointer,
    vertex_pointer: BatchBufferPointer,
    indices: BatchTypeArray,
    vertices: Vec<BatchTypeArray>,
    transforms: Vec<BatchTypeTransform>,
    order: i32,
}

impl BatchObject {
    pub fn new(
        index_pointer: BatchBufferPointer,
        vertex_pointer: BatchBufferPointer,
        indices: BatchTypeArray,
        vertices: Vec<BatchTypeArray>,
        transforms: Vec<BatchTypeTransform>,
        order: i32,
    ) -> Self {
        BatchObject {
            index_pointer,
            vertex_pointer,
            indices,
            vertices,
            transforms,
            order,
        }
    }

    pub fn order(&self) -> i32 {
        self.order
    }

    pub fn index_pointer(&self) -> BatchBufferPointer {
        self.index_pointer
    }

    pub fn vertex_pointer(&self) -> BatchBufferPointer {
        self.vertex_pointer
    }

    pub fn indices(&self) -> &BatchTypeArray {
        &self.indices
    }

    pub fn vertices(&self) -> &Vec<BatchTypeArray> {
        &self.vertices
    }

    pub fn set_transform(&mut self, attribute: BatchAttributeIndex, transform: BatchTypeTransform) {
        self.transforms[attribute as usize] = transform;
    }

    pub fn set_indices(&mut self, data: BatchTypeArray) {
        self.indices = data;
    }

    pub fn set_vertices(&mut self, attribute: BatchAttributeIndex, data: BatchTypeArray) {
        self.vertices[attribute as usize] = data;
    }

    pub fn set_index_pointer(&mut self, pointer: BatchBufferPointer) {
        self.index_pointer = pointer;
    }

    pub fn set_vertex_pointer(&mut self, pointer: BatchBufferPointer) {
        self.vertex_pointer = pointer;
    }

    pub fn get_transformed_vertices(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<BatchTypeArray, BatchObjectError> {
        let vertices = self.vertices.get(attribute as usize).ok_or_else(|| {
            BatchObjectError::AttributeReferenceToDataOutOfRange {
                attribute,
                data_len: self.vertices.len(),
            }
        })?;
        if let Some(transform) = self.transforms.get(attribute as usize) {
            return vertices
                .transform(transform)
                .map_err(BatchObjectError::InvalidArrayType);
        }
        Ok(vertices.clone())
    }

    pub fn get_v1u32_indices(&self) -> Result<Vec<u32>, BatchObjectError> {
        match &self.indices {
            BatchTypeArray::V1U32 { data } => Ok(data
                .iter()
                .map(|index| *index + self.vertex_pointer.first() as u32)
                .collect()),
            x => Err(BatchObjectError::InvalidArrayType(
                BatchTypeArrayError::InvalidTransformType {
                    expect: "v1u32",
                    actual: x.label(),
                },
            )),
        }
    }

    pub fn get_v1f32_vertices(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<f32>, BatchObjectError> {
        match self.get_transformed_vertices(attribute)? {
            BatchTypeArray::V1F32 { data } => Ok(data),
            x => Err(BatchObjectError::InvalidArrayType(
                BatchTypeArrayError::InvalidTransformType {
                    expect: "v1f32",
                    actual: x.label(),
                },
            )),
        }
    }

    pub fn get_v1u32_vertices(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<u32>, BatchObjectError> {
        match self.get_transformed_vertices(attribute)? {
            BatchTypeArray::V1U32 { data } => Ok(data),
            x => Err(BatchObjectError::InvalidArrayType(
                BatchTypeArrayError::InvalidTransformType {
                    expect: "v1u32",
                    actual: x.label(),
                },
            )),
        }
    }

    pub fn get_v2f32_vertices(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec2<f32>>, BatchObjectError> {
        match self.get_transformed_vertices(attribute)? {
            BatchTypeArray::V2F32 { data } => Ok(data),
            x => Err(BatchObjectError::InvalidArrayType(
                BatchTypeArrayError::InvalidTransformType {
                    expect: "v2f32",
                    actual: x.label(),
                },
            )),
        }
    }

    pub fn get_v2u32_vertices(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec2<u32>>, BatchObjectError> {
        match self.get_transformed_vertices(attribute)? {
            BatchTypeArray::V2U32 { data } => Ok(data),
            x => Err(BatchObjectError::InvalidArrayType(
                BatchTypeArrayError::InvalidTransformType {
                    expect: "v2u32",
                    actual: x.label(),
                },
            )),
        }
    }

    pub fn get_v3f32_vertices(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec3<f32>>, BatchObjectError> {
        match self.get_transformed_vertices(attribute)? {
            BatchTypeArray::V3F32 { data } => Ok(data),
            x => Err(BatchObjectError::InvalidArrayType(
                BatchTypeArrayError::InvalidTransformType {
                    expect: "v3f32",
                    actual: x.label(),
                },
            )),
        }
    }

    pub fn get_v3u32_vertices(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec3<u32>>, BatchObjectError> {
        match self.get_transformed_vertices(attribute)? {
            BatchTypeArray::V3U32 { data } => Ok(data),
            x => Err(BatchObjectError::InvalidArrayType(
                BatchTypeArrayError::InvalidTransformType {
                    expect: "v3u32",
                    actual: x.label(),
                },
            )),
        }
    }

    pub fn get_v4f32_vertices(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec4<f32>>, BatchObjectError> {
        match self.get_transformed_vertices(attribute)? {
            BatchTypeArray::V4F32 { data } => Ok(data),
            x => Err(BatchObjectError::InvalidArrayType(
                BatchTypeArrayError::InvalidTransformType {
                    expect: "v4f32",
                    actual: x.label(),
                },
            )),
        }
    }

    pub fn get_v4u32_vertices(
        &self,
        attribute: BatchAttributeIndex,
    ) -> Result<Vec<TVec4<u32>>, BatchObjectError> {
        match self.get_transformed_vertices(attribute)? {
            BatchTypeArray::V4U32 { data } => Ok(data),
            x => Err(BatchObjectError::InvalidArrayType(
                BatchTypeArrayError::InvalidTransformType {
                    expect: "v4u32",
                    actual: x.label(),
                },
            )),
        }
    }
}

#[derive(Debug)]
pub enum BatchObjectError {
    AttributeReferenceToDataOutOfRange { attribute: u32, data_len: usize },
    InvalidArrayType(BatchTypeArrayError),
}
