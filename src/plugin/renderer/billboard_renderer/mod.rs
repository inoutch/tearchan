use crate::batch::batch_billboard::{BatchBillboard, BatchBillboardProvider};
use crate::plugin::renderer::billboard_renderer::billboard_command_queue::BillboardCommandQueue;
use crate::plugin::renderer::billboard_renderer::billboard_render_object::BillboardRenderObject;
use crate::plugin::renderer::billboard_renderer::billboard_renderer_default_provider::BillboardRendererDefaultProvider;
use crate::plugin::renderer::billboard_renderer::billboard_renderer_provider::BillboardRendererProvider;
use serde::export::Option::Some;
use tearchan_core::game::game_cast_manager::GameCastManager;
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_object_caster::{GameObjectCaster, GameObjectCasterType};
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::GameObject;
use tearchan_graphics::hal::backend::{RenderBundle, RendererContext, Texture};

pub mod billboard_command_queue;
pub mod billboard_render_object;
pub mod billboard_renderer_default_provider;
pub mod billboard_renderer_provider;

pub struct BillboardRenderer<T: BillboardRendererProvider> {
    object_manager: GameObjectManager<dyn BillboardRenderObject>,
    batch: BatchBillboard,
    provider: T,
    cast_manager: GameCastManager,
}

impl<T: BillboardRendererProvider> BillboardRenderer<T> {
    pub fn new(render_bundle: &RenderBundle, provider: T) -> BillboardRenderer<T> {
        BillboardRenderer {
            object_manager: GameObjectManager::new(),
            batch: BatchBillboardProvider::new(render_bundle),
            provider,
            cast_manager: GameCastManager::default(),
        }
    }

    pub fn provider(&self) -> &T {
        &self.provider
    }

    pub fn provider_mut(&mut self) -> &mut T {
        &mut self.provider
    }

    pub fn register_caster_for_billboard(
        &mut self,
        caster: GameObjectCasterType<dyn BillboardRenderObject>,
    ) {
        self.cast_manager.register(GameObjectCaster::new(caster));
    }
}

impl BillboardRenderer<BillboardRendererDefaultProvider> {
    pub fn from_texture(r: &mut RendererContext, texture: Texture) -> Self {
        BillboardRenderer {
            object_manager: GameObjectManager::new(),
            batch: BatchBillboardProvider::new(r.render_bundle()),
            provider: BillboardRendererDefaultProvider::from_texture(r, texture),
            cast_manager: GameCastManager::default(),
        }
    }
}

impl<T: BillboardRendererProvider> GamePlugin for BillboardRenderer<T> {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut billboard_object) = self
            .cast_manager
            .cast::<dyn BillboardRenderObject>(game_object)
        {
            billboard_object
                .borrow_mut()
                .attach_queue(BillboardCommandQueue::new(self.batch.create_queue()));
            self.object_manager.add(billboard_object);
        }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(mut billboard_object) = self
            .cast_manager
            .cast::<dyn BillboardRenderObject>(game_object)
        {
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
