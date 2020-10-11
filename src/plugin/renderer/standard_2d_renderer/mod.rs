use crate::batch::batch2d::{Batch2D, Batch2DProvider};
use crate::plugin::object::camera::Camera2DObject;
use crate::plugin::renderer::standard_2d_renderer::standard_2d_render_object::Standard2DRenderObject;
use crate::plugin::renderer::standard_2d_renderer::standard_2d_renderer_default_provider::Standard2DRendererDefaultProvider;
use crate::plugin::renderer::standard_2d_renderer::standard_2d_renderer_provider::Standard2DRendererProvider;
use serde::export::Option::Some;
use tearchan_core::game::game_cast_manager::GameCastManager;
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_object_caster::{GameObjectCaster, GameObjectCasterType};
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
use tearchan_graphics::hal::backend::{RendererContext, Texture};

pub mod standard_2d_render_object;
pub mod standard_2d_renderer_default_provider;
pub mod standard_2d_renderer_provider;

pub struct Standard2DRenderer<T: Standard2DRendererProvider> {
    provider: T,
    object_manager: GameObjectManager<dyn Standard2DRenderObject>,
    camera_object: Option<GameObject<dyn Camera2DObject>>,
    camera_label: String,
    batch2d: Batch2D,
    cast_manager: GameCastManager,
}

impl<T: Standard2DRendererProvider> Standard2DRenderer<T> {
    pub fn new(
        r: &mut RendererContext,
        provider: T,
        camera_label: String,
    ) -> Standard2DRenderer<T> {
        let batch2d = Batch2DProvider::new(r.render_bundle());

        Standard2DRenderer {
            provider,
            object_manager: GameObjectManager::new(),
            camera_object: None,
            camera_label,
            batch2d,
            cast_manager: GameCastManager::default(),
        }
    }

    pub fn create_batch_queue(&mut self) -> BatchCommandQueue {
        self.batch2d.create_queue()
    }

    pub fn register_caster_for_render_object(
        &mut self,
        caster: GameObjectCasterType<dyn Standard2DRenderObject>,
    ) {
        self.cast_manager.register(GameObjectCaster::new(caster));
    }

    pub fn register_caster_for_camera(&mut self, caster: GameObjectCasterType<dyn Camera2DObject>) {
        self.cast_manager.register(GameObjectCaster::new(caster));
    }
}

impl Standard2DRenderer<Standard2DRendererDefaultProvider> {
    pub fn from_texture(
        r: &mut RendererContext,
        texture: Texture,
        camera_label: String,
    ) -> Standard2DRenderer<Standard2DRendererDefaultProvider> {
        let batch2d = Batch2DProvider::new(r.render_bundle());
        Standard2DRenderer {
            provider: Standard2DRendererDefaultProvider::from_texture(r, texture),
            object_manager: GameObjectManager::new(),
            camera_object: None,
            camera_label,
            batch2d,
            cast_manager: GameCastManager::default(),
        }
    }
}

impl<T: Standard2DRendererProvider> GamePlugin for Standard2DRenderer<T> {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut render_object) = self
            .cast_manager
            .cast::<dyn Standard2DRenderObject>(game_object)
        {
            render_object
                .borrow_mut()
                .attach_queue(self.batch2d.create_queue());
            self.object_manager.add(render_object);
        }

        if let Some(camera_object) = self.cast_manager.cast::<dyn Camera2DObject>(game_object) {
            if self.camera_label == camera_object.borrow().label() {
                self.camera_object = Some(camera_object);
            }
        }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        self.object_manager.remove(&game_object.id());

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
        self.batch2d.flush();
        self.provider.prepare(context, camera_object.camera2d());

        context.r.draw_elements(
            self.provider.graphic_pipeline(),
            self.batch2d.provider().index_count(),
            self.batch2d.provider().index_buffer(),
            &self.batch2d.provider().vertex_buffers(),
        );
    }
}
