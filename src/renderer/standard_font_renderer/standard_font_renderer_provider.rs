use tearchan_core::game::game_context::GameContext;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::{GraphicPipeline, Texture};
use tearchan_graphics::shader::standard_2d_shader_program::Standard2DShaderProgram;

pub trait StandardFontRendererProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline;

    fn prepare(&mut self, context: &mut GameContext, texture: &Texture);
}

pub struct StandardFontRendererDefaultProvider {
    camera: Camera2D,
    shader_program: Standard2DShaderProgram,
    graphic_pipeline: GraphicPipeline,
}

impl StandardFontRendererDefaultProvider {
    pub fn new(
        camera: Camera2D,
        shader_program: Standard2DShaderProgram,
        graphic_pipeline: GraphicPipeline,
    ) -> Self {
        StandardFontRendererDefaultProvider {
            camera,
            shader_program,
            graphic_pipeline,
        }
    }
}

impl StandardFontRendererProvider for StandardFontRendererDefaultProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline {
        &self.graphic_pipeline
    }

    fn prepare(&mut self, context: &mut GameContext, texture: &Texture) {
        self.camera.update();
        self.shader_program.prepare(self.camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        self.shader_program
            .create_write_descriptor_sets(descriptor_set, texture)
            .write(context.r.render_bundle());
    }
}
