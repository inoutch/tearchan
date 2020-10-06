use crate::batch::batch_billboard::{BatchBillboard, BatchBillboardProvider};
use crate::plugin::renderer::billboard_renderer::billboard_command_queue::BillboardCommandQueue;
use crate::plugin::renderer::billboard_renderer::billboard_object::BillboardObject;
use crate::plugin::renderer::billboard_renderer::billboard_renderer_default_provider::BillboardRendererDefaultProvider;
use crate::plugin::renderer::billboard_renderer::billboard_renderer_provider::BillboardRendererProvider;
use serde::export::Option::Some;
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::hal::backend::{RenderBundle, RendererContext, Texture};

pub mod billboard_command_queue;
pub mod billboard_object;
pub mod billboard_renderer_default_provider;
pub mod billboard_renderer_provider;

pub struct BillboardRenderer<T: BillboardRendererProvider> {
    object_manager: GameObjectManager<dyn BillboardObject>,
    batch: BatchBillboard,
    provider: T,
}

impl<T: BillboardRendererProvider> BillboardRenderer<T> {
    pub fn new(render_bundle: &RenderBundle, provider: T) -> BillboardRenderer<T> {
        BillboardRenderer {
            object_manager: GameObjectManager::new(),
            batch: BatchBillboardProvider::new(render_bundle),
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

impl BillboardRenderer<BillboardRendererDefaultProvider> {
    pub fn from_texture(r: &mut RendererContext, texture: Texture) -> Self {
        BillboardRenderer {
            object_manager: GameObjectManager::new(),
            batch: BatchBillboardProvider::new(r.render_bundle()),
            provider: BillboardRendererDefaultProvider::from_texture(r, texture),
        }
    }
}

impl<T: BillboardRendererProvider> GamePlugin for BillboardRenderer<T> {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut billboard_object) = game_object.cast::<dyn BillboardObject>() {
            billboard_object
                .borrow_mut()
                .attach_queue(BillboardCommandQueue::new(self.batch.create_queue()));
            self.object_manager.add(billboard_object);
        }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut billboard_object) = game_object.cast::<dyn BillboardObject>() {
            billboard_object.borrow_mut().detach();
            self.object_manager.remove(&game_object.id());
        }
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
