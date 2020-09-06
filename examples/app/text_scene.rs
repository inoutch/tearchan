use nalgebra_glm::vec2;
use tearchan::core::graphic::batch::batch2d::Batch2D;
use tearchan::core::graphic::camera_2d::Camera2D;
use tearchan::core::graphic::font_texture::FontTexture;
use tearchan::core::graphic::hal::backend::GraphicPipeline;
use tearchan::core::graphic::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan::core::graphic::polygon::polygon_2d::Polygon2DInterface;
use tearchan::core::graphic::polygon::text_label::TextLabel;
use tearchan::core::graphic::polygon::Polygon;
use tearchan::core::graphic::shader::standard_2d_shader_program::Standard2DShaderProgram;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::extension::shared::make_shared;
use tearchan::math::mesh::MeshBuilder;

pub struct TextScene {
    camera: Camera2D,
    batch: Batch2D,
    shader_program: Standard2DShaderProgram,
    graphic_pipeline: GraphicPipeline,
    font_texture: FontTexture,
    text_label: TextLabel,
}

impl TextScene {
    pub fn creator() -> SceneCreator {
        |ctx, _| Box::new(TextScene::new(ctx))
    }

    pub fn new(ctx: &mut SceneContext) -> Self {
        let camera = Camera2D::new(&ctx.graphics.display_size().logical);
        let shader_program = Standard2DShaderProgram::new(ctx.graphics, camera.base());
        let graphic_pipeline = ctx
            .graphics
            .create_graphic_pipeline(shader_program.shader(), GraphicPipelineConfig::default());
        let mut batch = Batch2D::new_batch2d(ctx.graphics);

        let ttf_bytes = include_bytes!("../data/fonts/GenShinGothic-Light.ttf");
        let font_texture = FontTexture::new(
            ctx.graphics,
            ttf_bytes.to_vec(),
            "ABCDあいうえお　ylpWAR 今日は\n良い天気ですねデスネ",
            100.0f32,
        )
        .unwrap();
        let mut text_label = TextLabel::new(&font_texture);
        text_label.set_anchor_point(vec2(0.00f32, 0.0f32));
        batch.add(&text_label.polygon(), 0);

        let mesh = MeshBuilder::new()
            .with_square(vec2(200.0f32, 200.0f32))
            .build()
            .unwrap();
        let polygon = make_shared(Polygon::new(mesh));
        batch.add(&polygon, 0);

        TextScene {
            camera,
            batch,
            shader_program,
            graphic_pipeline,
            font_texture,
            text_label,
        }
    }
}

impl SceneBase for TextScene {
    fn update(&mut self, ctx: &mut SceneContext, _delta: f32) {
        self.font_texture.flush(ctx.graphics);

        self.camera.update();
        self.batch.flush();

        self.shader_program.prepare(self.camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        let write_descriptor_sets = self
            .shader_program
            .create_write_descriptor_sets(descriptor_set, self.font_texture.texture());

        ctx.graphics.write_descriptor_sets(write_descriptor_sets);
        ctx.graphics.draw_elements(
            &self.graphic_pipeline,
            self.batch.index_size(),
            self.batch.index_buffer(),
            &self.batch.vertex_buffers(),
        );
    }

    fn on_character(&mut self, character: &char) {
        let text = format!("{}{}", self.font_texture.text(), character);
        self.font_texture.set_text(text);

        self.batch.remove(self.text_label.polygon());
        self.text_label = TextLabel::new(&self.font_texture);
        self.text_label.set_anchor_point(vec2(0.00f32, 0.0f32));
        self.batch.add(self.text_label.polygon(), 0);
    }
}
