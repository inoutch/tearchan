use crate::plugin::renderer::standard_3d_renderer::standard_3d_renderer_provider::Standard3DRendererProvider;
use nalgebra_glm::{vec3, Vec3};
use tearchan_core::game::game_context::GameContext;
use tearchan_graphics::camera::camera_3d::Camera3D;
use tearchan_graphics::hal::backend::{GraphicPipeline, RendererContext, Texture};
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::shader::standard_3d_shader_program::Standard3DShaderProgram;

pub struct Standard3DRendererDefaultProvider {
    texture: Texture,
    graphic_pipeline: GraphicPipeline,
    shader_program: Standard3DShaderProgram,
    light_position: Vec3,
    light_color: Vec3,
    ambient_strength: f32,
}

impl Standard3DRendererDefaultProvider {
    pub fn new(
        texture: Texture,
        graphic_pipeline: GraphicPipeline,
        shader_program: Standard3DShaderProgram,
        light_position: Vec3,
        light_color: Vec3,
        ambient_strength: f32,
    ) -> Self {
        Standard3DRendererDefaultProvider {
            texture,
            graphic_pipeline,
            shader_program,
            light_position,
            light_color,
            ambient_strength,
        }
    }

    pub fn from_texture(r: &mut RendererContext, texture: Texture) -> Self {
        let aspect =
            r.render_bundle().display_size().logical.x / r.render_bundle().display_size().logical.y;
        let camera = Camera3D::default_with_aspect(aspect);
        let shader_program = Standard3DShaderProgram::new(r.render_bundle(), camera.base());
        let graphic_pipeline = GraphicPipeline::new(
            r.render_bundle(),
            r.primary_render_pass(),
            shader_program.shader(),
            GraphicPipelineConfig::default(),
        );

        Standard3DRendererDefaultProvider::new(
            texture,
            graphic_pipeline,
            shader_program,
            vec3(0.0f32, 1.0f32, 0.0f32),
            vec3(1.0f32, 1.0f32, 1.0f32),
            0.1f32,
        )
    }
}

impl Standard3DRendererProvider for Standard3DRendererDefaultProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline {
        &self.graphic_pipeline
    }

    fn prepare(&mut self, context: &mut GameContext, camera: &Camera3D) {
        self.shader_program.prepare(
            camera.combine(),
            &self.light_position,
            &self.light_color,
            self.ambient_strength,
        );

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        self.shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture)
            .write(context.r.render_bundle());
    }
}
