use crate::plugin::renderer::billboard_renderer::billboard_renderer_provider::BillboardRendererProvider;
use tearchan_core::game::game_context::GameContext;
use tearchan_graphics::camera::camera_3d::Camera3D;
use tearchan_graphics::hal::backend::{GraphicPipeline, RendererContext, Texture};
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::shader::billboard_shader_program::BillboardShaderProgram;

pub struct BillboardRendererDefaultProvider {
    texture: Texture,
    camera: Camera3D,
    graphic_pipeline: GraphicPipeline,
    shader_program: BillboardShaderProgram,
}

impl BillboardRendererDefaultProvider {
    pub fn from_texture(r: &mut RendererContext, texture: Texture) -> Self {
        let aspect =
            r.render_bundle().display_size().logical.x / r.render_bundle().display_size().logical.y;
        let camera = Camera3D::default_with_aspect(aspect);
        let shader_program = BillboardShaderProgram::new(r.render_bundle(), camera.base());
        let graphic_pipeline = GraphicPipeline::new(
            r.render_bundle(),
            r.primary_render_pass(),
            shader_program.shader(),
            GraphicPipelineConfig::default(),
        );

        BillboardRendererDefaultProvider {
            texture,
            camera,
            graphic_pipeline,
            shader_program,
        }
    }

    pub fn camera_mut(&mut self) -> &mut Camera3D {
        &mut self.camera
    }
}

impl BillboardRendererProvider for BillboardRendererDefaultProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline {
        &self.graphic_pipeline
    }

    fn prepare(&mut self, context: &mut GameContext) {
        self.camera.update();
        self.shader_program.prepare(self.camera.base());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        self.shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture)
            .write(context.r.render_bundle());
    }
}
