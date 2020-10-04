use tearchan_core::game::game_context::GameContext;
use tearchan_graphics::hal::backend::GraphicPipeline;

pub trait Standard3DRendererProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline;

    fn prepare(&mut self, context: &mut GameContext);
}
