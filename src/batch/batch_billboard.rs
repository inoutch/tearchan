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

pub type BatchBillboard = Batch<BatchBillboardProvider>;
pub const BATCH_BILLBOARD_ATTRIB_IDX: u32 = 0;
pub const BATCH_BILLBOARD_ATTRIB_POS: u32 = 1;
pub const BATCH_BILLBOARD_ATTRIB_COL: u32 = 2;
pub const BATCH_BILLBOARD_ATTRIB_TEX: u32 = 3;
pub const BATCH_BILLBOARD_ATTRIB_OGN: u32 = 4;

pub struct BatchBillboardProvider {
    index_buffer: BatchBuffer<IndexBuffer>,
    position_buffer: BatchBuffer<VertexBuffer>,
    color_buffer: BatchBuffer<VertexBuffer>,
    texcoord_buffer: BatchBuffer<VertexBuffer>,
    origin_buffer: BatchBuffer<VertexBuffer>,
}

impl BatchBillboardProvider {
    pub fn new(render_bundle: &RenderBundle) -> BatchBillboard {
        Batch::new(BatchBillboardProvider {
            index_buffer: create_index_batch_buffer(render_bundle),
            position_buffer: create_vertex_batch_buffer(render_bundle),
            color_buffer: create_vertex_batch_buffer(render_bundle),
            texcoord_buffer: create_vertex_batch_buffer(render_bundle),
            origin_buffer: create_vertex_batch_buffer(render_bundle),
        })
    }

    pub fn index_buffer(&self) -> &IndexBuffer {
        self.index_buffer.buffer()
    }

    pub fn vertex_buffers(&self) -> Vec<&VertexBuffer> {
        vec![
            self.position_buffer.buffer(),
            self.color_buffer.buffer(),
            self.texcoord_buffer.buffer(),
            self.origin_buffer.buffer(),
        ]
    }

    pub fn index_count(&self) -> usize {
        self.index_buffer.last()
    }
}

impl BatchProvider for BatchBillboardProvider {
    fn run(&mut self, command: BatchProviderCommand) {
        match &command {
            BatchProviderCommand::Add { id, data, .. } => {
                debug_assert_eq!(
                    data[BATCH_BILLBOARD_ATTRIB_POS as usize].len(),
                    data[BATCH_BILLBOARD_ATTRIB_COL as usize].len()
                );
                debug_assert_eq!(
                    data[BATCH_BILLBOARD_ATTRIB_COL as usize].len(),
                    data[BATCH_BILLBOARD_ATTRIB_TEX as usize].len()
                );
                debug_assert_eq!(
                    data[BATCH_BILLBOARD_ATTRIB_TEX as usize].len(),
                    data[BATCH_BILLBOARD_ATTRIB_OGN as usize].len()
                );

                self.index_buffer
                    .allocate(*id, data[BATCH_BILLBOARD_ATTRIB_IDX as usize].len());
                self.position_buffer
                    .allocate(*id, data[BATCH_BILLBOARD_ATTRIB_POS as usize].len() * 3);
                self.color_buffer
                    .allocate(*id, data[BATCH_BILLBOARD_ATTRIB_COL as usize].len() * 4);
                self.texcoord_buffer
                    .allocate(*id, data[BATCH_BILLBOARD_ATTRIB_TEX as usize].len() * 2);
                self.origin_buffer
                    .allocate(*id, data[BATCH_BILLBOARD_ATTRIB_OGN as usize].len() * 3);
            }
            BatchProviderCommand::Remove { id } => {
                self.index_buffer.free(*id);
                self.position_buffer.free(*id);
                self.color_buffer.free(*id);
                self.texcoord_buffer.free(*id);
                self.origin_buffer.free(*id);
            }
            BatchProviderCommand::Replace {
                id,
                attribute,
                data,
            } => match *attribute {
                BATCH_BILLBOARD_ATTRIB_IDX => self.index_buffer.reallocate(*id, data.len()),
                BATCH_BILLBOARD_ATTRIB_POS => self.position_buffer.reallocate(*id, data.len() * 3),
                BATCH_BILLBOARD_ATTRIB_COL => self.color_buffer.reallocate(*id, data.len() * 4),
                BATCH_BILLBOARD_ATTRIB_TEX => self.texcoord_buffer.reallocate(*id, data.len() * 2),
                BATCH_BILLBOARD_ATTRIB_OGN => self.origin_buffer.reallocate(*id, data.len() * 3),
                _ => {}
            },
            _ => {}
        }
    }

    fn sort(&mut self, ids: Vec<u64>) -> HashSet<u32, RandomState> {
        self.index_buffer.sort(&ids);
        self.position_buffer.sort(&ids);
        self.color_buffer.sort(&ids);
        self.texcoord_buffer.sort(&ids);
        self.origin_buffer.sort(&ids);
        let mut set = HashSet::with_capacity(5);
        set.insert(BATCH_BILLBOARD_ATTRIB_IDX);
        set.insert(BATCH_BILLBOARD_ATTRIB_POS);
        set.insert(BATCH_BILLBOARD_ATTRIB_COL);
        set.insert(BATCH_BILLBOARD_ATTRIB_TEX);
        set.insert(BATCH_BILLBOARD_ATTRIB_OGN);
        set
    }

    fn flush(&mut self, batch_object_manager: &mut BatchObjectManager) {
        let index_buffer = &self.index_buffer;
        let position_buffer = &self.position_buffer;
        let color_buffer = &self.color_buffer;
        let texcoord_buffer = &self.texcoord_buffer;
        let normal_buffer = &self.origin_buffer;

        let mut index_mapping = index_buffer.buffer().open(0, index_buffer.len());
        let mut position_mapping = position_buffer.buffer().open(0, position_buffer.len());
        let mut color_mapping = color_buffer.buffer().open(0, color_buffer.len());
        let mut texcoord_mapping = texcoord_buffer.buffer().open(0, texcoord_buffer.len());
        let mut origin_mapping = normal_buffer.buffer().open(0, normal_buffer.len());

        batch_object_manager.flush(|object, attribute| match attribute {
            BATCH_BILLBOARD_ATTRIB_IDX => {
                let p0 = index_buffer.get_pointer(&object.id).unwrap();
                let p1 = position_buffer.get_pointer(&object.id).unwrap();
                object.for_each_v1u32(BATCH_BILLBOARD_ATTRIB_IDX as usize, |i, v| {
                    index_mapping.set(v + p1.first as u32 / 3, i + p0.first);
                });
            }
            BATCH_BILLBOARD_ATTRIB_POS => {
                let p1 = position_buffer.get_pointer(&object.id).unwrap();
                object.for_each_v3f32(BATCH_BILLBOARD_ATTRIB_POS as usize, |i, v| {
                    position_mapping.set(v, i + p1.first);
                });
            }
            BATCH_BILLBOARD_ATTRIB_COL => {
                let p2 = color_buffer.get_pointer(&object.id).unwrap();
                object.for_each_v4f32(BATCH_BILLBOARD_ATTRIB_COL as usize, |i, v| {
                    color_mapping.set(v, i + p2.first);
                });
            }
            BATCH_BILLBOARD_ATTRIB_TEX => {
                let p3 = texcoord_buffer.get_pointer(&object.id).unwrap();
                object.for_each_v2f32(BATCH_BILLBOARD_ATTRIB_TEX as usize, |i, v| {
                    texcoord_mapping.set(v, i + p3.first);
                });
            }
            BATCH_BILLBOARD_ATTRIB_OGN => {
                let p4 = normal_buffer.get_pointer(&object.id).unwrap();
                object.for_each_v3f32(BATCH_BILLBOARD_ATTRIB_OGN as usize, |i, v| {
                    origin_mapping.set(v, i + p4.first);
                });
            }
            _ => {}
        });

        self.index_buffer.buffer().close(index_mapping);
        self.position_buffer.buffer().close(position_mapping);
        self.color_buffer.buffer().close(color_mapping);
        self.texcoord_buffer.buffer().close(texcoord_mapping);
        self.origin_buffer.buffer().close(origin_mapping);
    }
}
