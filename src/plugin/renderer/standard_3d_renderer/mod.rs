use crate::batch::batch3d::{Batch3D, Batch3DProvider};
use crate::plugin::renderer::standard_3d_renderer::standard_3d_object::Standard3DObject;
use crate::plugin::renderer::standard_3d_renderer::standard_3d_renderer_default_provider::Standard3DRendererDefaultProvider;
use crate::plugin::renderer::standard_3d_renderer::standard_3d_renderer_provider::Standard3DRendererProvider;
use serde::export::Option::Some;
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::hal::backend::{RenderBundle, RendererContext, Texture};

pub mod standard_3d_object;
pub mod standard_3d_renderer_default_provider;
pub mod standard_3d_renderer_provider;

pub struct Standard3DRenderer<T: Standard3DRendererProvider> {
    object_manager: GameObjectManager<dyn Standard3DObject>,
    batch: Batch3D,
    provider: T,
}

impl<T: Standard3DRendererProvider> Standard3DRenderer<T> {
    pub fn new(render_bundle: &RenderBundle, provider: T) -> Standard3DRenderer<T> {
        Standard3DRenderer {
            object_manager: GameObjectManager::new(),
            batch: Batch3DProvider::new(render_bundle),
            provider,
        }
    }

    pub fn provider(&self) -> &T {
        &self.provider
    }

    pub fn provider_mut(&mut self) -> &mut T {
        &mut self.provider
    }
}

impl Standard3DRenderer<Standard3DRendererDefaultProvider> {
    pub fn from_texture(
        r: &mut RendererContext,
        texture: Texture,
    ) -> Standard3DRenderer<Standard3DRendererDefaultProvider> {
        Standard3DRenderer {
            object_manager: GameObjectManager::new(),
            batch: Batch3DProvider::new(r.render_bundle()),
            provider: Standard3DRendererDefaultProvider::from_texture(r, texture),
        }
    }
}

impl<T: Standard3DRendererProvider> GamePlugin for Standard3DRenderer<T> {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut object) = game_object.cast::<dyn Standard3DObject>() {
            object.borrow_mut().attach_queue(self.batch.create_queue());
            self.object_manager.add(object);
        }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        self.object_manager.remove(&game_object.id());
    }

    fn on_update(&mut self, context: &mut GameContext) {
        self.batch.flush();
        self.provider.prepare(context);

        context.r.draw_elements(
            self.provider.graphic_pipeline(),
            self.batch.provider().index_count(),
            self.batch.provider().index_buffer(),
            &self.batch.provider().vertex_buffers(),
        );
    }
}
