use crate::batch::batch_billboard::{
    BATCH_BILLBOARD_ATTRIB_COL, BATCH_BILLBOARD_ATTRIB_POS, BATCH_BILLBOARD_ATTRIB_TEX,
};
use crate::plugin::renderer::sprite_renderer::sprite::Sprite;
use tearchan_core::game::object::EMPTY_ID;
use tearchan_graphics::batch::batch_command::{
    BatchCommand, BatchCommandData, BatchCommandTransform, BatchObjectId,
};
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_utility::mesh::square::create_square_colors;
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

        let id = self
            .batch_queue
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
            .unwrap();
        self.update_sprite(id, sprite);
        id
    }

    pub fn update_sprite(&mut self, id: BatchObjectId, sprite: &Sprite) {
        sprite.update_frame(|positions, texcoords| {
            self.batch_queue.queue(BatchCommand::Replace {
                id,
                attribute: BATCH_BILLBOARD_ATTRIB_POS,
                data: BatchCommandData::V3F32 { data: positions },
            });
            self.batch_queue.queue(BatchCommand::Replace {
                id,
                attribute: BATCH_BILLBOARD_ATTRIB_TEX,
                data: BatchCommandData::V2F32 { data: texcoords },
            });
        });
        sprite.update_transform(|transform| {
            self.batch_queue.queue(BatchCommand::Transform {
                id,
                attribute: BATCH_BILLBOARD_ATTRIB_POS,
                transform: BatchCommandTransform::Mat4 { m: transform },
            });
        });
        sprite.update_color(|color| {
            self.batch_queue.queue(BatchCommand::Replace {
                id,
                attribute: BATCH_BILLBOARD_ATTRIB_COL,
                data: BatchCommandData::V4F32 {
                    data: create_square_colors(color.clone_owned()),
                },
            });
        });
    }

    pub fn destroy_sprite(&mut self, id: BatchObjectId) {
        self.batch_queue.queue(BatchCommand::Remove { id });
    }
}
