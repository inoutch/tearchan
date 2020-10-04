use crate::plugin::renderer::sprite_renderer::sprite::Sprite;
use tearchan_core::game::object::EMPTY_ID;
use tearchan_graphics::batch::batch_command::{BatchCommand, BatchCommandData, BatchObjectId};
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_utility::math::vec::vec2_zero;
use tearchan_utility::mesh::square::{
    create_square_positions_from_frame, create_square_texcoords_from_frame,
};
use tearchan_utility::mesh::MeshBuilder;

pub struct SpriteCommandQueue {
    batch_queue: BatchCommandQueue,
}

impl SpriteCommandQueue {
    pub fn new(batch_queue: BatchCommandQueue) -> Self {
        SpriteCommandQueue { batch_queue }
    }

    pub fn create_sprite(&mut self, sprite: &Sprite, order: Option<i32>) -> BatchObjectId {
        let frame = sprite
            .texture_atlas()
            .frames
            .first()
            .expect("There must be at least one or more frames");
        let (indices, positions, colors, texcoords, _) = MeshBuilder::new()
            .with_frame(sprite.texture_atlas().size.to_vec2(), frame)
            .build()
            .unwrap()
            .decompose();

        self.batch_queue
            .queue(BatchCommand::Add {
                id: EMPTY_ID,
                data: vec![
                    BatchCommandData::V1U32 { data: indices },
                    BatchCommandData::V3F32 { data: positions },
                    BatchCommandData::V4F32 { data: colors },
                    BatchCommandData::V2F32 { data: texcoords },
                ],
                order,
            })
            .unwrap()
    }

    pub fn update_sprite(&mut self, id: BatchObjectId, sprite: &Sprite) {
        if let Some(frame) = sprite.current_frame() {
            let positions = create_square_positions_from_frame(&vec2_zero(), frame);
            let texcoords =
                create_square_texcoords_from_frame(sprite.texture_atlas().size.to_vec2(), frame);
            self.batch_queue.queue(BatchCommand::Replace {
                id,
                attribute: 1,
                data: BatchCommandData::V3F32 { data: positions },
            });
            self.batch_queue.queue(BatchCommand::Replace {
                id,
                attribute: 3,
                data: BatchCommandData::V2F32 { data: texcoords },
            });
        }
    }

    pub fn destroy_sprite(&mut self, id: BatchObjectId) {
        self.batch_queue.queue(BatchCommand::Remove { id });
    }
}
