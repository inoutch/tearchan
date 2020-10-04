use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use tearchan_graphics::batch::batch_buffer::BatchBuffer;
use tearchan_graphics::batch::batch_command::BatchProviderCommand;
use tearchan_graphics::batch::batch_object_manager::BatchObjectManager;
use tearchan_graphics::batch::batch_provider::BatchProvider;
use tearchan_graphics::batch::Batch;
use tearchan_graphics::hal::backend::{IndexBuffer, RenderBundle, VertexBuffer};
use tearchan_graphics::hal::buffer::buffer_interface::{
    BufferInterface, BufferMappedMemoryInterface,
};
use tearchan_graphics::hal::helper::{create_index_batch_buffer, create_vertex_batch_buffer};

pub type Batch2D = Batch<Batch2DProvider>;

pub struct Batch2DProvider {
    index_buffer: BatchBuffer<IndexBuffer>,
    vertex_buffers: Vec<BatchBuffer<VertexBuffer>>,
}

impl Batch2DProvider {
    pub fn new(render_bundle: &RenderBundle) -> Batch2D {
        Batch::new(Batch2DProvider {
            index_buffer: create_index_batch_buffer(render_bundle),
            vertex_buffers: vec![
                create_vertex_batch_buffer(render_bundle),
                create_vertex_batch_buffer(render_bundle),
                create_vertex_batch_buffer(render_bundle),
            ],
        })
    }

    pub fn index_buffer(&self) -> &IndexBuffer {
        self.index_buffer.buffer()
    }

    pub fn vertex_buffers(&self) -> Vec<&VertexBuffer> {
        self.vertex_buffers.iter().map(|b| b.buffer()).collect()
    }

    pub fn index_count(&self) -> usize {
        self.index_buffer.last()
    }
}

impl BatchProvider for Batch2DProvider {
    fn run(&mut self, command: BatchProviderCommand) {
        match &command {
            BatchProviderCommand::Add { id, data, .. } => {
                debug_assert_eq!(data[1].len(), data[2].len());
                debug_assert_eq!(data[2].len(), data[3].len());

                self.index_buffer.allocate(*id, data[0].len());
                self.vertex_buffers[0].allocate(*id, data[1].len() * 3);
                self.vertex_buffers[1].allocate(*id, data[2].len() * 4);
                self.vertex_buffers[2].allocate(*id, data[3].len() * 2);
            }
            BatchProviderCommand::Remove { id } => {
                self.index_buffer.free(*id);
                self.vertex_buffers[0].free(*id);
                self.vertex_buffers[1].free(*id);
                self.vertex_buffers[2].free(*id);
            }
            BatchProviderCommand::Replace {
                id,
                attribute,
                data,
            } => match attribute {
                0 => self.index_buffer.reallocate(*id, data.len()),
                1 => self.vertex_buffers[0].reallocate(*id, data.len() * 3),
                2 => self.vertex_buffers[1].reallocate(*id, data.len() * 4),
                3 => self.vertex_buffers[2].reallocate(*id, data.len() * 2),
                _ => {}
            },
            _ => {}
        }
    }

    fn sort(&mut self, ids: Vec<u64>) -> HashSet<u32, RandomState> {
        self.index_buffer.sort(&ids);
        self.vertex_buffers[0].sort(&ids);
        self.vertex_buffers[1].sort(&ids);
        self.vertex_buffers[2].sort(&ids);
        let mut set = HashSet::with_capacity(4);
        set.insert(0);
        set.insert(1);
        set.insert(2);
        set.insert(3);
        set
    }

    fn flush(&mut self, batch_object_manager: &mut BatchObjectManager) {
        let mut index_mapping = self.index_buffer.buffer().open(0, self.index_buffer.len());
        let mut position_mapping = self.vertex_buffers[0]
            .buffer()
            .open(0, self.vertex_buffers[0].len());
        let mut color_mapping = self.vertex_buffers[1]
            .buffer()
            .open(0, self.vertex_buffers[1].len());
        let mut texcoord_mapping = self.vertex_buffers[2]
            .buffer()
            .open(0, self.vertex_buffers[2].len());
        let index_buffer = &self.index_buffer;
        let vertex_buffers = &self.vertex_buffers;
        batch_object_manager.flush(|object, attribute| match attribute {
            0 => {
                let p0 = index_buffer.get_pointer(&object.id).unwrap();
                let p1 = vertex_buffers[0].get_pointer(&object.id).unwrap();
                object.for_each_v1u32(0, |i, v| {
                    index_mapping.set(v + p1.first as u32 / 3, i + p0.first);
                });
            }
            1 => {
                let p1 = vertex_buffers[0].get_pointer(&object.id).unwrap();
                object.for_each_v3f32(1, |i, v| {
                    position_mapping.set(v, i + p1.first);
                });
            }
            2 => {
                let p2 = vertex_buffers[1].get_pointer(&object.id).unwrap();
                object.for_each_v4f32(2, |i, v| {
                    color_mapping.set(v, i + p2.first);
                });
            }
            3 => {
                let p3 = vertex_buffers[2].get_pointer(&object.id).unwrap();
                object.for_each_v2f32(3, |i, v| {
                    texcoord_mapping.set(v, i + p3.first);
                });
            }
            _ => {}
        });

        self.index_buffer.buffer().close(index_mapping);
        self.vertex_buffers[0].buffer().close(position_mapping);
        self.vertex_buffers[1].buffer().close(color_mapping);
        self.vertex_buffers[2].buffer().close(texcoord_mapping);
    }
}
