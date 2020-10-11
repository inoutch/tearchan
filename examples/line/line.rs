use nalgebra_glm::{translate, vec3, Mat4};
use std::ops::Range;
use tearchan::batch::batch_line::BATCH_LINE_ATTRIB_POS;
use tearchan::plugin::renderer::standard_line_renderer::standard_line_render_object::StandardLineRenderObject;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::EMPTY_ID;
use tearchan_graphics::batch::batch_command::{
    BatchCommand, BatchCommandData, BatchCommandTransform, BatchObjectId,
};
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_utility::mesh::MeshBuilder;

pub struct Line {
    batch_object_id: BatchObjectId,
    batch_queue: Option<BatchCommandQueue>,
}

impl Default for Line {
    fn default() -> Self {
        Line {
            batch_object_id: EMPTY_ID,
            batch_queue: None,
        }
    }
}

impl GameObjectBase for Line {}

impl StandardLineRenderObject for Line {
    fn attach_queue(&mut self, mut queue: BatchCommandQueue) {
        let (indices, positions, colors, _, _) = MeshBuilder::new()
            .with_grid(
                50.0f32,
                Range {
                    start: (-5, -5),
                    end: (5, 5),
                },
            )
            .build()
            .unwrap()
            .decompose();

        self.batch_object_id = queue
            .queue(BatchCommand::Add {
                id: self.batch_object_id,
                data: vec![
                    BatchCommandData::V1U32 { data: indices },
                    BatchCommandData::V3F32 { data: positions },
                    BatchCommandData::V4F32 { data: colors },
                ],
                order: None,
            })
            .unwrap();
        queue.queue(BatchCommand::Transform {
            id: self.batch_object_id,
            transform: BatchCommandTransform::Mat4 {
                m: translate(&Mat4::identity(), &vec3(300.0f32, 300.0f32, 0.0f32)),
            },
            attribute: BATCH_LINE_ATTRIB_POS,
        });
    }

    fn detach(&mut self) {
        if let Some(queue) = &mut self.batch_queue {
            queue.queue(BatchCommand::Remove {
                id: self.batch_object_id,
            });
        }
    }
}
