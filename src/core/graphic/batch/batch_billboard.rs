use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_provider::{
    close_buffers, open_buffers, BatchBufferContext, BatchProvider,
};
use crate::core::graphic::batch::helpers::{create_index_batch_buffer, create_vertex_batch_buffer};
use crate::core::graphic::batch::{Batch, BatchContext};
use crate::core::graphic::hal::backend::{IndexBuffer, RendererApi, VertexBuffer};
use crate::core::graphic::hal::index_buffer::IndexBufferMappedMemory;
use crate::core::graphic::hal::vertex_buffer::VertexBufferMappedMemory;
use crate::core::graphic::polygon::billboard::Billboard;
use crate::extension::shared::Shared;
use crate::math::mesh::IndexType;
use std::rc::Rc;

pub type BatchBillboard = Batch<Billboard, BatchBillboardProvider, IndexBuffer, VertexBuffer>;

impl BatchBillboard {
    pub fn new_batch_billboard(api: &RendererApi) -> BatchBillboard {
        Batch::new(BatchBillboardProvider::new(api))
    }
}

pub struct BatchBillboardProvider {
    index_buffer: BatchBuffer<IndexBuffer>,
    index_mapping: Option<IndexBufferMappedMemory>,
    vertex_buffers: Vec<BatchBufferContext<BatchBuffer<VertexBuffer>>>,
    vertex_mappings: Vec<VertexBufferMappedMemory>,
}

impl BatchBillboardProvider {
    pub fn new(api: &RendererApi) -> BatchBillboardProvider {
        BatchBillboardProvider {
            index_buffer: create_index_batch_buffer(api),
            index_mapping: None,
            vertex_buffers: vec![
                BatchBufferContext::new(create_vertex_batch_buffer(api), 3), // position
                BatchBufferContext::new(create_vertex_batch_buffer(api), 4), // colors
                BatchBufferContext::new(create_vertex_batch_buffer(api), 2), // texcoords
                BatchBufferContext::new(create_vertex_batch_buffer(api), 3), // origins
            ],
            vertex_mappings: vec![],
        }
    }
}

impl BatchProvider<Billboard, IndexBuffer, VertexBuffer> for BatchBillboardProvider {
    fn update(&mut self, context: &Rc<BatchContext<Billboard>>, force: bool) {
        debug_assert_eq!(
            self.vertex_buffers.len(),
            4,
            "Invalid vertex buffers length"
        );
        debug_assert_eq!(
            context.vertex_pointers.len(),
            4,
            "Invalid object pointers length"
        );

        // update positions, colors, texcoords, normals, indices
        let vertex_offset =
            context.vertex_pointers[0].borrow().first / self.vertex_buffers[0].stride;
        let mut object = context.object.borrow_mut();
        {
            let mut polygon = object.polygon().borrow_mut();
            let index_mapping = match &mut self.index_mapping {
                Some(mapping) => mapping,
                None => return,
            };
            polygon.copy_indices_into(
                index_mapping,
                context.index_pointer.borrow().first,
                vertex_offset as IndexType,
                force,
            );
            polygon.copy_positions_into(
                &mut self.vertex_mappings[0],
                context.vertex_pointers[0].borrow().first,
                force,
            );
            polygon.copy_colors_into(
                &mut self.vertex_mappings[1],
                context.vertex_pointers[1].borrow().first,
                force,
            );
            polygon.copy_texcoords_into(
                &mut self.vertex_mappings[2],
                context.vertex_pointers[2].borrow().first,
                force,
            );
        }
        object.copy_origins_into(
            &mut self.vertex_mappings[3],
            context.vertex_pointers[3].borrow().first,
            force,
        );
    }

    fn index_buffer(&self) -> &BatchBuffer<IndexBuffer> {
        &self.index_buffer
    }

    fn index_buffer_mut(&mut self) -> &mut BatchBuffer<IndexBuffer> {
        &mut self.index_buffer
    }

    fn index_size(&self, object: &Shared<Billboard>) -> usize {
        object.borrow().polygon().borrow().index_size()
    }

    fn vertex_buffer_contexts_mut(
        &mut self,
    ) -> &mut Vec<BatchBufferContext<BatchBuffer<VertexBuffer>>> {
        &mut self.vertex_buffers
    }

    fn vertex_buffer_contexts(&self) -> &Vec<BatchBufferContext<BatchBuffer<VertexBuffer>>> {
        &self.vertex_buffers
    }

    fn vertex_size(&self, object: &Shared<Billboard>) -> usize {
        object.borrow().polygon().borrow().vertex_size()
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
