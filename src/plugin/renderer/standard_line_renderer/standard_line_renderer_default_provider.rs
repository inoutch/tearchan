use crate::plugin::renderer::standard_line_renderer::standard_line_renderer_provider::StandardLineRendererProvider;
use gfx_hal::pso::{FrontFace, PolygonMode, Primitive, Rasterizer, State};
use nalgebra_glm::Mat4;
use tearchan_core::game::game_context::GameContext;
use tearchan_graphics::camera::Camera;
use tearchan_graphics::hal::backend::{GraphicPipeline, RendererContext};
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::shader::standard_line_shader_program::StandardLineShaderProgram;

pub struct StandardLineRendererDefaultProvider {
    graphic_pipeline: GraphicPipeline,
    shader_program: StandardLineShaderProgram,
}

impl StandardLineRendererDefaultProvider {
    pub fn new(r: &mut RendererContext) -> Self {
        let shader_program = StandardLineShaderProgram::new(r.render_bundle(), Mat4::identity());
        let graphic_pipeline = GraphicPipeline::new(
            r.render_bundle(),
            r.primary_render_pass(),
            shader_program.shader(),
            GraphicPipelineConfig {
                rasterizer: Rasterizer {
                    polygon_mode: PolygonMode::Line,
                    cull_face: gfx_hal::pso::Face::NONE,
                    front_face: FrontFace::CounterClockwise,
                    depth_clamping: false,
                    depth_bias: None,
                    conservative: false,
                    line_width: State::Static(1.0),
                },
                primitive: Primitive::LineList,
            },
        );
        StandardLineRendererDefaultProvider {
            graphic_pipeline,
            shader_program,
        }
    }
}

impl StandardLineRendererProvider for StandardLineRendererDefaultProvider {
    fn graphic_pipeline(&self) -> &GraphicPipeline {
        &self.graphic_pipeline
    }

    fn prepare(&mut self, context: &mut GameContext, camera: &Camera) {
        self.shader_program.prepare(camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        self.shader_program
            .create_write_descriptor_sets(descriptor_set)
            .write(context.r.render_bundle());
    }
}
