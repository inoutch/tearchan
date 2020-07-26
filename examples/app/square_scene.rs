use nalgebra_glm::{vec2, vec3, vec4, Vec2};
use palette::rgb::Rgb;
use palette::{ConvertInto, Hsv};
use serde::export::Option::Some;
use tearchan::core::graphic::batch::batch2d::Batch2D;
use tearchan::core::graphic::camera_2d::Camera2D;
use tearchan::core::graphic::hal::backend::{GraphicPipeline, Texture};
use tearchan::core::graphic::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan::core::graphic::hal::renderer::ResizeContext;
use tearchan::core::graphic::hal::texture::TextureConfig;
use tearchan::core::graphic::image::Image;
use tearchan::core::graphic::polygon::{Polygon, PolygonCommon};
use tearchan::core::graphic::shader::standard_2d_shader_program::Standard2DShaderProgram;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::core::scene::touch::Touch;
use tearchan::extension::shared::{clone_shared, make_shared, Shared};
use tearchan::math::mesh::MeshBuilder;
use winit::event::{KeyboardInput, VirtualKeyCode};

pub struct SquareScene {
    camera: Camera2D,
    batch: Batch2D,
    shader_program: Standard2DShaderProgram,
    texture: Texture,
    graphic_pipeline: GraphicPipeline,
    polygons: Vec<Shared<Polygon>>,
    screen_size: Vec2,
}

impl SquareScene {
    pub fn creator() -> SceneCreator {
        |context, _| {
            let screen_size = context.graphics.display_size().logical.clone_owned();
            let image = Image::new_empty();

            let mut camera = Camera2D::new(screen_size.clone_owned());
            camera.update();

            let texture = context
                .graphics
                .create_texture(&image, TextureConfig::default());

            let shader_program = Standard2DShaderProgram::new(context.graphics, camera.base());
            let graphic_pipeline = context
                .graphics
                .create_graphic_pipeline(shader_program.shader(), GraphicPipelineConfig::default());

            let mut batch = Batch2D::new_batch2d(context.graphics);
            {
                let mesh = MeshBuilder::new()
                    .with_square(vec2(100.0f32, 100.0f32))
                    .build()
                    .unwrap();
                let polygon = make_shared(Polygon::new(mesh));
                polygon.borrow_mut().set_position(vec3(
                    screen_size.x / 2.0f32,
                    screen_size.y / 2.0f32,
                    0.0f32,
                ));
                batch.add(&polygon, 0);
            }
            {
                let mesh = MeshBuilder::new()
                    .with_square(vec2(100.0f32, 100.0f32))
                    .build()
                    .unwrap();
                let polygon = make_shared(Polygon::new(mesh));
                polygon.borrow_mut().set_position(vec3(
                    screen_size.x / 2.0f32 - 100.0f32,
                    screen_size.y / 2.0f32 - 100.0f32,
                    0.0f32,
                ));
                batch.add(&polygon, 0);
            }

            Box::new(SquareScene {
                camera,
                batch,
                shader_program,
                graphic_pipeline,
                texture,
                polygons: vec![],
                screen_size,
            })
        }
    }
}

impl SceneBase for SquareScene {
    fn update(&mut self, scene_context: &mut SceneContext, _delta: f32) {
        self.camera.update();
        self.batch.flush();

        self.shader_program.prepare(self.camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        let write_descriptor_sets = self
            .shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture);

        scene_context
            .graphics
            .write_descriptor_sets(write_descriptor_sets);
        scene_context.graphics.draw_elements(
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

    fn on_key_down(&mut self, input: &KeyboardInput) {
        match input.virtual_keycode {
            None => {}
            Some(code) => match code {
                VirtualKeyCode::Right => {
                    let x: u32 = rand::random::<u32>() % (self.screen_size.x as u32 - 100u32);
                    let y: u32 = rand::random::<u32>() % (self.screen_size.y as u32 - 100u32);
                    let color: Rgb =
                        Hsv::new((rand::random::<u32>() % 360u32) as f32, 1.0f32, 1.0f32)
                            .convert_into();

                    let mesh = MeshBuilder::new()
                        .with_square(vec2(100.0f32, 100.0f32))
                        .build()
                        .unwrap();
                    let polygon = make_shared(Polygon::new(mesh));
                    polygon
                        .borrow_mut()
                        .set_position(vec3(x as f32, y as f32, 0.0f32));
                    polygon.borrow_mut().set_color(vec4(
                        color.red,
                        color.blue,
                        color.green,
                        1.0f32,
                    ));
                    self.batch.add(&polygon, 0);
                    self.polygons.push(clone_shared(&polygon));
                }
                VirtualKeyCode::Left => {
                    if self.polygons.is_empty() {
                        return;
                    }
                    let i = rand::random::<usize>() % self.polygons.len();
                    self.batch.remove(&self.polygons.remove(i));
                }
                _ => {}
            },
        }
    }

    fn on_key_up(&mut self, _input: &KeyboardInput) {}

    fn on_resize(&mut self, context: &mut ResizeContext) {
        self.camera = Camera2D::new(context.display_size.logical.clone_owned());
    }
}
