use crate::batch::batch_line::{BatchLine, BatchLineProvider};
use crate::plugin::object::camera::CameraObject;
use crate::plugin::renderer::standard_line_renderer::standard_line_render_object::StandardLineRenderObject;
use crate::plugin::renderer::standard_line_renderer::standard_line_renderer_default_provider::StandardLineRendererDefaultProvider;
use crate::plugin::renderer::standard_line_renderer::standard_line_renderer_provider::StandardLineRendererProvider;
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_graphics::hal::backend::RendererContext;

pub mod standard_line_render_object;
pub mod standard_line_renderer_default_provider;
pub mod standard_line_renderer_provider;

pub struct StandardLineRenderer<T: StandardLineRendererProvider> {
    provider: T,
    object_manager: GameObjectManager<dyn StandardLineRenderObject>,
    camera_object: Option<GameObject<dyn CameraObject>>,
    camera_label: String,
    batch: BatchLine,
}

pub struct Standard2DRenderer {}

impl<T: StandardLineRendererProvider> StandardLineRenderer<T> {
    pub fn from_provider(
        r: &mut RendererContext,
        provider: T,
        camera_label: String,
    ) -> StandardLineRenderer<T> {
        let batch = BatchLineProvider::new(r.render_bundle());

        StandardLineRenderer {
            provider,
            object_manager: GameObjectManager::new(),
            camera_object: None,
            camera_label,
            batch,
        }
    }

    pub fn create_batch_queue(&mut self) -> BatchCommandQueue {
        self.batch.create_queue()
    }
}

impl StandardLineRenderer<StandardLineRendererDefaultProvider> {
    pub fn new(r: &mut RendererContext, camera_label: String) -> Self {
        let provider = StandardLineRendererDefaultProvider::new(r);
        StandardLineRenderer::from_provider(r, provider, camera_label)
    }
}

impl<T: StandardLineRendererProvider> GamePlugin for StandardLineRenderer<T> {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut render_object) = game_object.cast::<dyn StandardLineRenderObject>() {
            render_object
                .borrow_mut()
                .attach_queue(self.batch.create_queue());
            self.object_manager.add(render_object);
        }

        if let Some(camera_object) = game_object.cast::<dyn CameraObject>() {
            if self.camera_label == camera_object.borrow().label() {
                self.camera_object = Some(camera_object);
            }
        }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut render_object) = game_object.cast::<dyn StandardLineRenderObject>() {
            render_object.borrow_mut().detach();
            self.object_manager.remove(&game_object.id());
        }

        if let Some(camera_object) = &self.camera_object {
            if camera_object.id() == game_object.id() {
                self.camera_object = None;
            }
        }
    }

    fn on_update(&mut self, context: &mut GameContext) {
        let camera_object = match &self.camera_object {
            None => return,
            Some(camera) => camera.borrow(),
        };
        self.batch.flush();
        self.provider.prepare(context, camera_object.camera());

        context.r.draw_elements(
            self.provider.graphic_pipeline(),
            self.batch.provider().index_count(),
            self.batch.provider().index_buffer(),
            &self.batch.provider().vertex_buffers(),
        );
    }
}
