use nalgebra_glm::vec2;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::camera::camera_3d::Camera3D;
use tearchan_graphics::hal::backend::{GraphicPipeline, IndexBuffer, Texture, VertexBuffer};
use tearchan_graphics::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;
use tearchan_graphics::shader::quad_shader_program::QuadShaderProgram;
use tearchan_graphics::shader::standard_3d_shader_program::Standard3DShaderProgram;
use tearchan_utility::mesh::MeshBuilder;

pub struct QuadScene {
    texture: Texture,
    shader_program: QuadShaderProgram,
    graphic_pipeline: GraphicPipeline,
    index_buffer: IndexBuffer,
    position_buffer: VertexBuffer,
    color_buffer: VertexBuffer,
    texcoord_buffer: VertexBuffer,
    once: bool,
}

impl QuadScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let image = Image::new_empty();
            let texture = Texture::new(ctx.g.r.render_bundle(), &image, TextureConfig::for_pixel());
            let shader_program = QuadShaderProgram::new(ctx.g.r.render_bundle());
            let graphics_pipeline = GraphicPipeline::new(
                ctx.g.r.render_bundle(),
                ctx.g.r.primary_render_pass(),
                shader_program.shader(),
                GraphicPipelineConfig::default(),
            );
            let (indices, positions, colors, texcoords, _) = MeshBuilder::new()
                .with_square(vec2(1.0f32, 1.0f32))
                .build()
                .unwrap()
                .decompose();
            let p = positions
                .iter()
                .map(|p| vec![p.x, p.y, p.z])
                .flatten()
                .collect::<Vec<f32>>();
            let c = colors
                .iter()
                .map(|p| vec![p.x, p.y, p.z, p.w])
                .flatten()
                .collect::<Vec<f32>>();
            let t = texcoords
                .iter()
                .map(|p| vec![p.x, p.y])
                .flatten()
                .collect::<Vec<f32>>();
            let index_buffer = IndexBuffer::new(ctx.g.r.render_bundle(), &indices);
            let position_buffer = VertexBuffer::new(ctx.g.r.render_bundle(), &p);
            let color_buffer = VertexBuffer::new(ctx.g.r.render_bundle(), &c);
            let texcoord_buffer = VertexBuffer::new(ctx.g.r.render_bundle(), &t);

            Box::new(QuadScene {
                texture,
                shader_program,
                graphic_pipeline: graphics_pipeline,
                index_buffer,
                position_buffer,
                color_buffer,
                texcoord_buffer,
                once: false,
            })
        }
    }
}

impl Scene for QuadScene {
    fn update(&mut self, context: &mut SceneContext) -> SceneResult {
        if self.once {
            return SceneResult::None;
        }
        self.once = true;
        let descriptor_set = self.graphic_pipeline.descriptor_set();
        self.shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture)
            .write(context.g.r.render_bundle());

        context.g.r.draw_elements(
            &self.graphic_pipeline,
            6,
            &self.index_buffer,
            &[
                &self.position_buffer,
                &self.color_buffer,
                &self.texcoord_buffer,
            ],
        );
        SceneResult::None
    }
}
