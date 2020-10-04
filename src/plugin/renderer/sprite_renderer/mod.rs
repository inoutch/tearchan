use crate::batch::batch2d::{Batch2D, Batch2DProvider};
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
    provider: T,
    object_manager: GameObjectManager<dyn SpriteObject>,
    batch2d: Batch2D,
}

impl<T> SpriteRenderer<T>
where
    T: Standard2DRendererProvider,
{
    pub fn new(r: &mut RendererContext, provider: T) -> Self {
        let batch2d = Batch2DProvider::new(r.render_bundle());

        SpriteRenderer {
            provider,
            object_manager: GameObjectManager::new(),
            batch2d,
        }
    }
}

impl SpriteRenderer<Standard2DRendererDefaultProvider> {
    pub fn from_texture(r: &mut RendererContext, texture: Texture) -> Self {
        let batch2d = Batch2DProvider::new(r.render_bundle());

        SpriteRenderer {
            provider: Standard2DRendererDefaultProvider::from_texture(r, texture),
            object_manager: GameObjectManager::new(),
            batch2d,
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
