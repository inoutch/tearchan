use tearchan::renderer::standard_2d_renderer::Standard2DRenderer;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::{GraphicPipeline, Texture};
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;
use tearchan_graphics::shader::standard_2d_shader_program::Standard2DShaderProgram;

pub struct CubeScene {}

impl CubeScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let image = Image::new_empty();
            let texture = Texture::new(ctx.g.r.render_bundle(), &image, TextureConfig::default());

            let plugin = Box::new(Standard2DRenderer::new(&mut ctx.g.r, texture));
            ctx.plugin_manager_mut()
                .add(plugin, "renderer".to_string(), 0);
            Box::new(CubeScene {})
        }
    }
}

impl Scene for CubeScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
