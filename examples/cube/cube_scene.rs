use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::GraphicPipeline;
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::shader::standard_2d_shader_program::Standard2DShaderProgram;

pub struct CubeScene {}

impl CubeScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let camera = Camera2D::new(&ctx.g.r.render_bundle().display_size().logical);
            let shader_program =
                Standard2DShaderProgram::new(ctx.g.r.render_bundle(), camera.base());
            let _ = GraphicPipeline::new(
                ctx.g.r.render_bundle(),
                ctx.g.r.primary_render_pass(),
                shader_program.shader(),
                GraphicPipelineConfig::default(),
            );
            Box::new(CubeScene {})
        }
    }
}

impl Scene for CubeScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
