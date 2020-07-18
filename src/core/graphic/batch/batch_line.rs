use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_provider::{
    close_buffers, open_buffers, BatchBufferContext, BatchProvider,
};
use crate::core::graphic::batch::helpers::{create_index_batch_buffer, create_vertex_batch_buffer};
use crate::core::graphic::batch::{Batch, BatchContext};
use crate::core::graphic::hal::backend::{IndexBuffer, RendererApi, VertexBuffer};
use crate::core::graphic::hal::index_buffer::IndexBufferMappedMemory;
use crate::core::graphic::hal::vertex_buffer::VertexBufferMappedMemory;
use crate::core::graphic::polygon::Polygon;
use crate::extension::shared::Shared;
use crate::math::mesh::IndexType;
use std::rc::Rc;

pub type BatchLine = Batch<Polygon, BatchLineProvider, IndexBuffer, VertexBuffer>;

pub struct BatchLineProvider {
    index_buffer: BatchBuffer<IndexBuffer>,
    index_mapping: Option<IndexBufferMappedMemory>,
    vertex_buffers: Vec<BatchBufferContext<BatchBuffer<VertexBuffer>>>,
    vertex_mappings: Vec<VertexBufferMappedMemory>,
}

impl BatchLine {
    pub fn new_batch_line(api: &RendererApi) -> BatchLine {
        Batch::new(BatchLineProvider::new(api))
    }
}

impl BatchLineProvider {
    pub fn new(api: &RendererApi) -> Self {
        BatchLineProvider {
            index_buffer: create_index_batch_buffer(api),
            index_mapping: None,
            vertex_buffers: vec![
                BatchBufferContext::new(create_vertex_batch_buffer(api), 3),
                BatchBufferContext::new(create_vertex_batch_buffer(api), 4),
            ],
            vertex_mappings: vec![],
        }
    }
}

impl BatchProvider<Polygon, IndexBuffer, VertexBuffer> for BatchLineProvider {
    fn update(&mut self, context: &Rc<BatchContext<Polygon>>, force: bool) {
        debug_assert_eq!(
            self.vertex_buffers.len(),
            2,
            "Invalid vertex buffers length"
        );
        debug_assert_eq!(
            context.vertex_pointers.len(),
            2,
            "Invalid object pointers length"
        );

        // update positions, colors, texcoords, normals, indices
        let vertex_offset =
            context.vertex_pointers[0].borrow().first / self.vertex_buffers[0].stride;
        let mut object = context.object.borrow_mut();
        let index_mapping = match &mut self.index_mapping {
            Some(mapping) => mapping,
            None => return,
        };
        object.copy_indices_into(
            index_mapping,
            context.index_pointer.borrow().first,
            vertex_offset as IndexType,
            force,
        );
        object.copy_positions_into(
            &mut self.vertex_mappings[0],
            context.vertex_pointers[0].borrow().first,
            force,
        );
        object.copy_colors_into(
            &mut self.vertex_mappings[1],
            context.vertex_pointers[1].borrow().first,
            force,
        );
    }

    fn index_buffer(&self) -> &BatchBuffer<IndexBuffer> {
        &self.index_buffer
    }

    fn index_buffer_mut(&mut self) -> &mut BatchBuffer<IndexBuffer> {
        &mut self.index_buffer
    }

    fn index_size(&self, object: &Shared<Polygon>) -> usize {
        object.borrow().index_size()
    }

    fn vertex_buffer_contexts_mut(
        &mut self,
    ) -> &mut Vec<BatchBufferContext<BatchBuffer<VertexBuffer>>> {
        &mut self.vertex_buffers
    }

    fn vertex_buffer_contexts(&self) -> &Vec<BatchBufferContext<BatchBuffer<VertexBuffer>>> {
        &self.vertex_buffers
    }

    fn vertex_size(&self, object: &Shared<Polygon>) -> usize {
        object.borrow().vertex_size()
    }

    fn open(&mut self) {
        open_buffers(
            &self.index_buffer,
            &mut self.index_mapping,
            &self.vertex_buffers,
            &mut self.vertex_mappings,
        );
    }

    fn close(&mut self) {
        close_buffers(
            &self.index_buffer,
            &mut self.index_mapping,
            &self.vertex_buffers,
            &mut self.vertex_mappings,
        );
    }
}
