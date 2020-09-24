use crate::batch::batch2d::{Batch2D, Batch2DProvider};
use nalgebra_glm::{vec2, vec3};
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::batch::batch_command::{BatchCommand, BatchCommandData, BATCH_ID_EMPTY};
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::{GraphicPipeline, RendererContext, Texture};
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::shader::standard_2d_shader_program::Standard2DShaderProgram;
use tearchan_utility::mesh::MeshBuilder;

pub trait Standard2DRenderObject: GameObjectBase {
    fn attach_queue(&mut self, queue: BatchCommandQueue);
}

pub struct Standard2DRenderer {
    texture: Texture,
    camera: Camera2D,
    shader_program: Standard2DShaderProgram,
    graphic_pipeline: GraphicPipeline,
    object_manager: GameObjectManager<dyn Standard2DRenderObject>,
    batch2d: Batch2D,
}

impl Standard2DRenderer {
    pub fn new(r: &mut RendererContext, texture: Texture) -> Standard2DRenderer {
        let camera = Camera2D::new(&r.render_bundle().display_size().logical);
        let shader_program = Standard2DShaderProgram::new(r.render_bundle(), camera.base());
        let graphic_pipeline = GraphicPipeline::new(
            r.render_bundle(),
            r.primary_render_pass(),
            shader_program.shader(),
            GraphicPipelineConfig::default(),
        );
        let mut batch2d = Batch2DProvider::new(r.render_bundle());
        let mut queue = batch2d.create_queue();
        let mesh = MeshBuilder::new()
            .with_square(vec2(100.0f32, 100.0f32))
            .build()
            .unwrap();
        queue.queue(BatchCommand::Add {
            id: BATCH_ID_EMPTY,
            data: vec![
                BatchCommandData::V3U32 {
                    data: vec![vec3(0u32, 3u32, 2u32), vec3(0u32, 1u32, 3u32)],
                },
                BatchCommandData::V3F32 {
                    data: mesh.positions.clone(),
                },
                BatchCommandData::V4F32 {
                    data: mesh.colors.clone(),
                },
                BatchCommandData::V2F32 {
                    data: mesh.texcoords.clone(),
                },
            ],
            order: None,
        });

        Standard2DRenderer {
            texture,
            camera,
            shader_program,
            graphic_pipeline,
            object_manager: GameObjectManager::new(),
            batch2d,
        }
    }
}

impl GamePlugin for Standard2DRenderer {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut render_object) = game_object.cast::<dyn Standard2DRenderObject>() {
            render_object
                .borrow_mut()
                .attach_queue(self.batch2d.create_queue());
            self.object_manager.add(render_object);
        }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        self.object_manager.remove(&game_object.id());
    }

    fn on_update(&mut self, context: &mut GameContext) {
        self.camera.update();
        self.batch2d.flush();

        self.shader_program.prepare(self.camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        self.shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture)
            .write(context.r.render_bundle());

        context.r.draw_elements(
            &self.graphic_pipeline,
            3,
            self.batch2d.provider().index_buffer(),
            &self.batch2d.provider().vertex_buffers(),
        );
    }
}
