use crate::app::texture_bundle::generate_window_texture_bundle;
use gfx_hal::image::Filter;
use nalgebra_glm::{vec2, vec3};
use tearchan::core::graphic::batch::batch2d::Batch2D;
use tearchan::core::graphic::camera_2d::Camera2D;
use tearchan::core::graphic::hal::backend::{GraphicPipeline, Texture};
use tearchan::core::graphic::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan::core::graphic::hal::renderer::ResizeContext;
use tearchan::core::graphic::hal::texture::TextureConfig;
use tearchan::core::graphic::polygon::polygon_2d::Polygon2DInterface;
use tearchan::core::graphic::polygon::sprite_atlas_window::SpriteAtlasWindow;
use tearchan::core::graphic::polygon::PolygonCommon;
use tearchan::core::graphic::shader::standard_2d_shader_program::Standard2DShaderProgram;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::core::scene::touch::Touch;
use tearchan::math::vec::make_vec2_zero;
use winit::event::KeyboardInput;

pub struct SpriteWindowScene {
    camera: Camera2D,
    batch: Batch2D,
    shader_program: Standard2DShaderProgram,
    graphic_pipeline: GraphicPipeline,
    texture: Texture,
}

impl SpriteWindowScene {
    pub fn creator() -> SceneCreator {
        |ctx, _| {
            let (texture_atlas, image) = generate_window_texture_bundle();
            let texture = ctx.graphics.create_texture(
                &image,
                TextureConfig {
                    filter: Filter::Nearest,
                    ..TextureConfig::default()
                },
            );

            let screen_size = &ctx.graphics.display_size().logical;
            let camera = Camera2D::new(screen_size);

            let shader_program = Standard2DShaderProgram::new(ctx.graphics, camera.base());
            let graphic_pipeline = ctx
                .graphics
                .create_graphic_pipeline(shader_program.shader(), GraphicPipelineConfig::default());
            let mut batch = Batch2D::new_batch2d(ctx.graphics);

            let mut sprite = SpriteAtlasWindow::new(
                texture_atlas,
                [
                    "window_0.png".to_string(),
                    "window_1.png".to_string(),
                    "window_2.png".to_string(),
                    "window_3.png".to_string(),
                    "window_4.png".to_string(),
                    "window_5.png".to_string(),
                    "window_6.png".to_string(),
                    "window_7.png".to_string(),
                    "window_8.png".to_string(),
                ],
                vec2(200.0f32, 100.0f32),
            );
            sprite.set_anchor_point(make_vec2_zero());
            sprite
                .polygon()
                .borrow_mut()
                .set_scale(vec3(4.0f32, 4.0f32, 1.0f32));
            batch.add(sprite.polygon(), 0);

            Box::new(SpriteWindowScene {
                camera,
                batch,
                shader_program,
                graphic_pipeline,
                texture,
            })
        }
    }
}

impl SceneBase for SpriteWindowScene {
    fn update(&mut self, ctx: &mut SceneContext, _delta: f32) {
        self.camera.update();
        self.batch.flush();

        self.shader_program.prepare(self.camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        let write_descriptor_sets = self
            .shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture);

        ctx.graphics.write_descriptor_sets(write_descriptor_sets);
        ctx.graphics.draw_elements(
            &self.graphic_pipeline,
            self.batch.index_size(),
            self.batch.index_buffer(),
            &self.batch.vertex_buffers(),
        );
    }

    fn on_touch_start(&mut self, _touch: &Touch) {}

    fn on_touch_end(&mut self, _touch: &Touch) {}

    fn on_touch_move(&mut self, _touch: &Touch) {}

    fn on_touch_cancel(&mut self, _touch: &Touch) {}

    fn on_key_down(&mut self, _input: &KeyboardInput) {}

    fn on_key_up(&mut self, _input: &KeyboardInput) {}

    fn on_resize(&mut self, _context: &mut ResizeContext) {}
}
