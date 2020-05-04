use gfx::state::CullFace;
use gfx_hal::pso::Factor::OneMinusSrcAlpha;
use gfx_hal::pso::{BlendOp, Factor, PolygonMode};

#[derive(Builder)]
#[builder(default)]
pub struct GraphicPipelineConfig {
    depth_test: bool,
    cull_face: CullFace,
    polygon_mode: PolygonMode,
    blend_op: BlendOp,
}

impl Default for GraphicPipelineConfig {
    fn default() -> Self {
        GraphicPipelineConfig {
            depth_test: true,
            cull_face: CullFace::Front,
            polygon_mode: PolygonMode::Fill,
            blend_op: BlendOp::Add {
                src: Factor::SrcAlpha,
                dst: OneMinusSrcAlpha,
            },
        }
    }
}
