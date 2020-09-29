use crate::batch::batch2d::{Batch2D, Batch2DProvider};
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::{GraphicPipeline, RendererContext, Texture};
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::shader::standard_2d_shader_program::Standard2DShaderProgram;

pub trait Standard2DRenderObject: GameObjectBase {
    fn attach_queue(&mut self, queue: BatchCommandQueue);
}

pub trait Standard2DRendererProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline;
    fn prepare(&mut self, context: &mut GameContext);
}

pub struct Standard2DRendererDefaultProvider {
    texture: Texture,
    camera: Camera2D,
    graphic_pipeline: GraphicPipeline,
    shader_program: Standard2DShaderProgram,
}

impl Standard2DRendererProvider for Standard2DRendererDefaultProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline {
        &self.graphic_pipeline
    }

    fn prepare(&mut self, context: &mut GameContext) {
        self.camera.update();
        self.shader_program.prepare(self.camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        self.shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture)
            .write(context.r.render_bundle());
    }
}

pub struct Standard2DRenderer<T: Standard2DRendererProvider> {
    provider: T,
    object_manager: GameObjectManager<dyn Standard2DRenderObject>,
    batch2d: Batch2D,
}

impl<T: Standard2DRendererProvider> Standard2DRenderer<T> {
    pub fn new(r: &mut RendererContext, provider: T) -> Standard2DRenderer<T> {
        let batch2d = Batch2DProvider::new(r.render_bundle());

        Standard2DRenderer {
            provider,
            object_manager: GameObjectManager::new(),
            batch2d,
        }
    }
}

impl Standard2DRenderer<Standard2DRendererDefaultProvider> {
    pub fn from_texture(
        r: &mut RendererContext,
        texture: Texture,
    ) -> Standard2DRenderer<Standard2DRendererDefaultProvider> {
        let camera = Camera2D::new(&r.render_bundle().display_size().logical);
        let shader_program = Standard2DShaderProgram::new(r.render_bundle(), camera.base());
        let graphic_pipeline = GraphicPipeline::new(
            r.render_bundle(),
            r.primary_render_pass(),
            shader_program.shader(),
            GraphicPipelineConfig::default(),
        );
        let batch2d = Batch2DProvider::new(r.render_bundle());

        Standard2DRenderer {
            provider: Standard2DRendererDefaultProvider {
                camera,
                shader_program,
                graphic_pipeline,
                texture,
            },
            object_manager: GameObjectManager::new(),
            batch2d,
        }
    }
}

impl<T: Standard2DRendererProvider> Standard2DRenderer<T> {
    pub fn create_batch_queue(&mut self) -> BatchCommandQueue {
        self.batch2d.create_queue()
    }
}

impl<T: Standard2DRendererProvider> GamePlugin for Standard2DRenderer<T> {
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
        self.batch2d.flush();
        self.provider.prepare(context);

        context.r.draw_elements(
            self.provider.graphic_pipeline(),
            self.batch2d.provider().index_count(),
            self.batch2d.provider().index_buffer(),
            &self.batch2d.provider().vertex_buffers(),
        );
    }
}
