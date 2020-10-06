use tearchan_core::game::game_context::GameContext;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::GraphicPipeline;

pub trait Standard2DRendererProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline;

    fn prepare(&mut self, context: &mut GameContext, camera: &Camera2D);
}
