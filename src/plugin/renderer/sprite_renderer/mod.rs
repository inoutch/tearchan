use crate::batch::batch2d::{Batch2D, Batch2DProvider};
use crate::plugin::object::camera::Camera2DObject;
use crate::plugin::renderer::sprite_renderer::sprite_command_queue::SpriteCommandQueue;
use crate::plugin::renderer::sprite_renderer::sprite_object::SpriteObject;
use crate::plugin::renderer::standard_2d_renderer::standard_2d_renderer_default_provider::Standard2DRendererDefaultProvider;
use crate::plugin::renderer::standard_2d_renderer::standard_2d_renderer_provider::Standard2DRendererProvider;
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::hal::backend::{RendererContext, Texture};

pub mod sprite;
pub mod sprite_command_queue;
pub mod sprite_object;

pub struct SpriteRenderer<T> {
    object_manager: GameObjectManager<dyn SpriteObject>,
    batch2d: Batch2D,
    camera_object: Option<GameObject<dyn Camera2DObject>>,
    camera_label: String,
    provider: T,
}

impl<T> SpriteRenderer<T>
where
    T: Standard2DRendererProvider,
{
    pub fn new(r: &mut RendererContext, provider: T, camera_label: String) -> Self {
        let batch2d = Batch2DProvider::new(r.render_bundle());

        SpriteRenderer {
            object_manager: GameObjectManager::new(),
            batch2d,
            camera_object: None,
            camera_label,
            provider,
        }
    }
}

impl SpriteRenderer<Standard2DRendererDefaultProvider> {
    pub fn from_texture(r: &mut RendererContext, texture: Texture, camera_label: String) -> Self {
        let batch2d = Batch2DProvider::new(r.render_bundle());

        SpriteRenderer {
            object_manager: GameObjectManager::new(),
            batch2d,
            camera_object: None,
            camera_label,
            provider: Standard2DRendererDefaultProvider::from_texture(r, texture),
        }
    }
}

impl<T> GamePlugin for SpriteRenderer<T>
where
    T: Standard2DRendererProvider,
{
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut render_object) = game_object.cast::<dyn SpriteObject>() {
            render_object
                .borrow_mut()
                .attach_sprite_queue(SpriteCommandQueue::new(self.batch2d.create_queue()));
            self.object_manager.add(render_object);
        }

        if let Some(camera_object) = game_object.cast::<dyn Camera2DObject>() {
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
        self.provider.prepare(context, camera_object.camera());

        context.r.draw_elements(
            self.provider.graphic_pipeline(),
            self.batch2d.provider().index_count(),
            self.batch2d.provider().index_buffer(),
            &self.batch2d.provider().vertex_buffers(),
        );
    }
}
