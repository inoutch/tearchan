use crate::plugin::renderer::standard_2d_renderer::standard_2d_renderer_provider::Standard2DRendererProvider;
use tearchan_core::game::game_context::GameContext;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::{GraphicPipeline, RendererContext, Texture};
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::shader::standard_2d_shader_program::Standard2DShaderProgram;

pub struct Standard2DRendererDefaultProvider {
    texture: Texture,
    camera: Camera2D,
    graphic_pipeline: GraphicPipeline,
    shader_program: Standard2DShaderProgram,
}

impl Standard2DRendererDefaultProvider {
    pub fn from_texture(r: &mut RendererContext, texture: Texture) -> Self {
        let camera = Camera2D::new(&r.render_bundle().display_size().logical);
        let shader_program = Standard2DShaderProgram::new(r.render_bundle(), camera.base());
        let graphic_pipeline = GraphicPipeline::new(
            r.render_bundle(),
            r.primary_render_pass(),
            shader_program.shader(),
            GraphicPipelineConfig::default(),
        );
        Standard2DRendererDefaultProvider {
            texture,
            camera,
            graphic_pipeline,
            shader_program,
        }
    }
}

impl Standard2DRendererProvider for Standard2DRendererDefaultProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline {
        &self.graphic_pipeline
    }

    fn prepare(&mut self, context: &mut GameContext) {
        self.camera.update();
        self.shader_program.prepare(self.camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        self.shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture)
            .write(context.r.render_bundle());
    }
}
