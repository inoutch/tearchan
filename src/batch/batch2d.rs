use tearchan_graphics::batch::batch_buffer::BatchBuffer;
use tearchan_graphics::batch::batch_command::BatchCommand;
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
    object_manager: BatchObjectManager,
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
            object_manager: BatchObjectManager::new(),
        })
    }
}

impl BatchProvider for Batch2DProvider {
    fn run(&mut self, command: BatchCommand) {
        match &command {
            BatchCommand::Add { id, data, .. } => {
                self.index_buffer.allocate(*id, data[0].len() * 3);
                self.vertex_buffers[0].allocate(*id, data[1].len() * 3);
                self.vertex_buffers[1].allocate(*id, data[2].len() * 4);
                self.vertex_buffers[2].allocate(*id, data[3].len() * 2);
            }
            BatchCommand::Remove { id } => {
                self.index_buffer.free(*id);
                self.vertex_buffers[0].free(*id);
                self.vertex_buffers[1].free(*id);
                self.vertex_buffers[2].free(*id);
            }
            BatchCommand::Replace {
                id,
                attribute,
                data,
            } => match attribute {
                0 => self.index_buffer.reallocate(*id, data.len() * 3),
                1 => self.vertex_buffers[0].reallocate(*id, data.len() * 3),
                2 => self.vertex_buffers[1].reallocate(*id, data.len() * 4),
                3 => self.vertex_buffers[2].reallocate(*id, data.len() * 2),
                _ => {}
            },
            _ => {}
        }
        self.object_manager.run(command);
    }

    fn flush(&mut self) {
        let mut index_mapping = self.index_buffer.buffer().open(0, self.index_buffer.size());
        let mut position_mapping = self.vertex_buffers[0]
            .buffer()
            .open(0, self.vertex_buffers[0].size());
        let mut color_mapping = self.vertex_buffers[1]
            .buffer()
            .open(0, self.vertex_buffers[1].size());
        let mut texcoord_mapping = self.vertex_buffers[2]
            .buffer()
            .open(0, self.vertex_buffers[2].size());
        let object_manager = &mut self.object_manager;
        let index_buffer = &self.index_buffer;
        let vertex_buffers = &self.vertex_buffers;
        object_manager.flush(|object| {
            let p0 = index_buffer.get_pointer(&object.id).unwrap();
            object.for_each_v3u32(0, |i, v| {
                index_mapping.set(v, i + p0.first);
            });
            let p1 = vertex_buffers[0].get_pointer(&object.id).unwrap();
            object.for_each_v3f32(1, |i, v| {
                position_mapping.set(v, i + p1.first);
            });
            let p2 = vertex_buffers[1].get_pointer(&object.id).unwrap();
            object.for_each_v4f32(2, |i, v| {
                color_mapping.set(v, i + p2.first);
            });
            let p3 = vertex_buffers[2].get_pointer(&object.id).unwrap();
            object.for_each_v2f32(3, |i, v| {
                texcoord_mapping.set(v, i + p3.first);
            });
        });

        self.index_buffer.buffer().close(index_mapping);
        self.vertex_buffers[0].buffer().close(position_mapping);
        self.vertex_buffers[1].buffer().close(color_mapping);
        self.vertex_buffers[2].buffer().close(texcoord_mapping);
    }
}
