use crate::batch::batch2d::{Batch2D, Batch2DProvider};
use crate::plugin::renderer::standard_font_renderer::standard_font_command::StandardFontCommand;
use crate::plugin::renderer::standard_font_renderer::standard_font_command_queue::StandardFontCommandQueue;
use crate::plugin::renderer::standard_font_renderer::standard_font_render_object::StandardFontRenderObject;
use crate::plugin::renderer::standard_font_renderer::standard_font_renderer_provider::{
    StandardFontRendererDefaultProvider, StandardFontRendererProvider,
};
use std::sync::mpsc::{channel, Receiver, Sender};
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::batch::batch_command::{BatchCommand, BatchCommandData};
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::{FontTexture, GraphicPipeline, RendererContext};
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::shader::standard_2d_shader_program::Standard2DShaderProgram;

pub mod standard_font_command;
pub mod standard_font_command_queue;
pub mod standard_font_render_object;
pub mod standard_font_renderer_provider;

pub struct StandardFontRenderer<T: StandardFontRendererProvider> {
    font_texture: FontTexture,
    object_manager: GameObjectManager<dyn StandardFontRenderObject>,
    batch2d: Batch2D,
    sender: Sender<StandardFontCommand>,
    receiver: Receiver<StandardFontCommand>,
    provider: T,
}

impl<T: StandardFontRendererProvider> StandardFontRenderer<T> {
    pub fn new(
        r: &mut RendererContext,
        font_texture: FontTexture,
        provider: T,
    ) -> StandardFontRenderer<T> {
        let (sender, receiver) = channel();
        StandardFontRenderer {
            font_texture,
            object_manager: GameObjectManager::new(),
            batch2d: Batch2DProvider::new(r.render_bundle()),
            sender,
            receiver,
            provider,
        }
    }
}

impl StandardFontRenderer<StandardFontRendererDefaultProvider> {
    pub fn from_font_texture(
        r: &mut RendererContext,
        font_texture: FontTexture,
    ) -> StandardFontRenderer<StandardFontRendererDefaultProvider> {
        let camera = Camera2D::new(&r.render_bundle().display_size().logical);
        let shader_program = Standard2DShaderProgram::new(r.render_bundle(), camera.base());
        let graphic_pipeline = GraphicPipeline::new(
            r.render_bundle(),
            r.primary_render_pass(),
            shader_program.shader(),
            GraphicPipelineConfig::default(),
        );

        StandardFontRenderer::new(
            r,
            font_texture,
            StandardFontRendererDefaultProvider::new(camera, shader_program, graphic_pipeline),
        )
    }
}

impl<T: StandardFontRendererProvider> StandardFontRenderer<T> {
    pub fn create_batch_queue(&mut self) -> StandardFontCommandQueue {
        StandardFontCommandQueue::new(self.batch2d.create_queue(), Sender::clone(&self.sender))
    }
}

impl<T: StandardFontRendererProvider> GamePlugin for StandardFontRenderer<T> {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut render_object) = game_object.cast::<dyn StandardFontRenderObject>() {
            render_object
                .borrow_mut()
                .attach_queue(StandardFontCommandQueue::new(
                    self.batch2d.create_queue(),
                    Sender::clone(&self.sender),
                ));
            self.object_manager.add(render_object);
        }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        self.object_manager.remove(&game_object.id());
    }

    fn on_update(&mut self, context: &mut GameContext) {
        self.batch2d.flush();

        let mut changed = false;
        while let Ok(command) = self.receiver.try_recv() {
            match command {
                StandardFontCommand::SetText { id, text } => {
                    self.font_texture.register_characters(&text);
                    let (mesh, _) = self.font_texture.create_mesh(&text);
                    let mut queue = self.batch2d.create_queue();
                    queue.queue(BatchCommand::Replace {
                        id,
                        attribute: 0,
                        data: BatchCommandData::V1U32 {
                            data: mesh.indices.clone(),
                        },
                    });
                    queue.queue(BatchCommand::Replace {
                        id,
                        attribute: 1,
                        data: BatchCommandData::V3F32 {
                            data: mesh.positions.clone(),
                        },
                    });
                    queue.queue(BatchCommand::Replace {
                        id,
                        attribute: 2,
                        data: BatchCommandData::V4F32 {
                            data: mesh.colors.clone(),
                        },
                    });
                    queue.queue(BatchCommand::Replace {
                        id,
                        attribute: 3,
                        data: BatchCommandData::V2F32 {
                            data: mesh.texcoords.clone(),
                        },
                    });
                    changed = true;
                }
            }
        }
        if changed {
            self.batch2d.flush();
        }

        self.provider.prepare(context, self.font_texture.texture());

        context.r.draw_elements(
            self.provider.graphic_pipeline(),
            self.batch2d.provider().index_count(),
            self.batch2d.provider().index_buffer(),
            &self.batch2d.provider().vertex_buffers(),
        );
    }
}
