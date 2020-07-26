use nalgebra_glm::{vec3, vec4};
use std::f32::consts::PI;
use std::io::BufReader;
use tearchan::core::graphic::batch::batch3d::Batch3D;
use tearchan::core::graphic::camera_3d::Camera3D;
use tearchan::core::graphic::hal::backend::{GraphicPipeline, Texture};
use tearchan::core::graphic::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan::core::graphic::hal::renderer::ResizeContext;
use tearchan::core::graphic::hal::texture::TextureConfig;
use tearchan::core::graphic::image::Image;
use tearchan::core::graphic::polygon::{Polygon, PolygonCommon};
use tearchan::core::graphic::shader::standard_3d_shader_program::Standard3DShaderProgram;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::core::scene::touch::Touch;
use tearchan::extension::shared::make_shared;
use tearchan::math::mesh::MeshBuilder;
use winit::event::KeyboardInput;

pub struct ObjScene {
    camera: Camera3D,
    camera_angle: f32,
    batch: Batch3D,
    shader_program: Standard3DShaderProgram,
    texture: Texture,
    graphic_pipeline: GraphicPipeline,
}

impl ObjScene {
    pub fn creator() -> SceneCreator {
        |ctx, _| {
            let screen_size = &ctx.graphics.display_size().logical;
            let image = Image::new_empty();

            let mut camera = Camera3D::default_with_aspect(screen_size.x / screen_size.y);
            camera.position = vec3(0.0f32, 2.0f32, 4.0f32);
            camera.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            camera.up = vec3(0.0f32, 1.0f32, 0.0f32);
            camera.update();

            let texture = ctx
                .graphics
                .create_texture(&image, TextureConfig::default());

            let shader_program = Standard3DShaderProgram::new(ctx.graphics, camera.base());
            let graphic_pipeline = ctx
                .graphics
                .create_graphic_pipeline(shader_program.shader(), GraphicPipelineConfig::default());

            let monkey_obj = include_str!("../data/obj/monkey.obj");
            let mut reader = BufReader::new(monkey_obj.as_bytes());
            let (models, _) = tobj::load_obj_buf(&mut reader, true, |p| {
                match p.file_name().unwrap().to_str().unwrap() {
                    "monkey.mtl" => {
                        let monkey_mtl = include_str!("../data/obj/monkey.mtl");
                        let mut reader = BufReader::new(monkey_mtl.as_bytes());
                        tobj::load_mtl_buf(&mut reader)
                    }
                    _ => unreachable!(),
                }
            })
            .unwrap();

            let mesh = MeshBuilder::new().with_model(&models[0]).build().unwrap();
            let mut batch = Batch3D::new_batch3d(ctx.graphics);
            let polygon = make_shared(Polygon::new(mesh));
            polygon
                .borrow_mut()
                .set_rotation_axis(vec3(0.0f32, 1.0f32, 0.0f32));
            polygon
                .borrow_mut()
                .set_color(vec4(1.0f32, 0.5f32, 0.5f32, 1.0f32));
            batch.add(&polygon, 0);

            Box::new(ObjScene {
                camera,
                camera_angle: 0.0f32,
                batch,
                shader_program,
                graphic_pipeline,
                texture,
            })
        }
    }
}

impl SceneBase for ObjScene {
    fn update(&mut self, ctx: &mut SceneContext, _delta: f32) {
        self.camera_angle += 1.0f32;
        self.camera.position = vec3(
            4.0f32 * (self.camera_angle / 180.0f32 * PI).cos(),
            2.0f32,
            4.0f32 * (self.camera_angle / 180.0f32 * PI).sin(),
        );
        self.camera.update();
        self.batch.flush();

        self.shader_program.prepare(
            self.camera.combine(),
            &self.camera.position,
            &vec3(1.0f32, 1.0f32, 1.0f32),
            0.2f32,
        );

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        let write_descriptor_sets = self
            .shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture);

        ctx.graphics
            .write_descriptor_sets(write_descriptor_sets);
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
