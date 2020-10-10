use intertrait::cast_to;
use nalgebra_glm::Mat4;
use serde::export::Option::Some;
use tearchan::batch::batch3d::{BATCH_3D_ATTRIB_NOM, BATCH_3D_ATTRIB_POS};
use tearchan::plugin::renderer::standard_3d_renderer::standard_3d_render_object::Standard3DRenderObject;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::EMPTY_ID;
use tearchan_graphics::batch::batch_command::{
    BatchCommand, BatchCommandData, BatchCommandTransform, BatchObjectId,
};
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_utility::mesh::MeshBuilder;
use tearchan_utility::rect::rect3;

pub trait TransformObject: GameObjectBase {
    fn set_transform(&mut self, transform: Mat4);
}

pub struct CubeObject {
    batch_object_id: BatchObjectId,
    batch_queue: Option<BatchCommandQueue>,
}

impl Default for CubeObject {
    fn default() -> Self {
        CubeObject {
            batch_object_id: EMPTY_ID,
            batch_queue: None,
        }
    }
}

#[cast_to]
impl GameObjectBase for CubeObject {}

#[cast_to]
impl Standard3DRenderObject for CubeObject {
    fn attach_queue(&mut self, mut queue: BatchCommandQueue) {
        let (indices, positions, colors, texcoords, normals) = MeshBuilder::new()
            .with_cube(&rect3(-0.1f32, -0.1f32, -0.1f32, 0.2f32, 0.2f32, 0.2f32))
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
                    BatchCommandData::V2F32 { data: texcoords },
                    BatchCommandData::V3F32 { data: normals },
                ],
                order: None,
            })
            .unwrap();
        self.batch_queue = Some(queue);
    }
}

#[cast_to]
impl TransformObject for CubeObject {
    fn set_transform(&mut self, transform: Mat4) {
        if let Some(queue) = &mut self.batch_queue {
            queue.queue(BatchCommand::Transform {
                id: self.batch_object_id,
                attribute: BATCH_3D_ATTRIB_POS,
                transform: BatchCommandTransform::Mat4 {
                    m: transform.clone_owned(),
                },
            });
            queue.queue(BatchCommand::Transform {
                id: self.batch_object_id,
                attribute: BATCH_3D_ATTRIB_NOM,
                transform: BatchCommandTransform::Mat4 { m: transform },
            });
        }
    }
}
