use intertrait::cast_to;
use tearchan::plugin::renderer::standard_3d_renderer::standard_3d_render_object::Standard3DRenderObject;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::EMPTY_ID;
use tearchan_graphics::batch::batch_command::{BatchCommand, BatchCommandData};
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_utility::mesh::MeshBuilder;
use tearchan_utility::rect::rect3;

#[derive(Default)]
pub struct GroundObject {}

#[cast_to]
impl GameObjectBase for GroundObject {}

#[cast_to]
impl Standard3DRenderObject for GroundObject {
    fn attach_queue(&mut self, mut queue: BatchCommandQueue) {
        let (indices, positions, colors, texcoords, normals) = MeshBuilder::new()
            .with_cube(&rect3(-1.0f32, -0.1f32, -1.0f32, 2.0f32, 0.2f32, 2.0f32))
            .build()
            .unwrap()
            .decompose();
        queue.queue(BatchCommand::Add {
            id: EMPTY_ID,
            data: vec![
                BatchCommandData::V1U32 { data: indices },
                BatchCommandData::V3F32 { data: positions },
                BatchCommandData::V4F32 { data: colors },
                BatchCommandData::V2F32 { data: texcoords },
                BatchCommandData::V3F32 { data: normals },
            ],
            order: None,
        });
    }
}
