use nalgebra_glm::vec2;
use tearchan::core::graphic::batch::batch2d::Batch2D;
use tearchan::core::graphic::batch::batch_buffer_f32::BatchBufferF32;
use tearchan::core::graphic::batch::default::Batch;
use tearchan::core::graphic::camera_2d::Camera2D;
use tearchan::core::graphic::hal::backend::{FixedGraphicPipeline, FixedTexture};
use tearchan::core::graphic::hal::image::Image;
use tearchan::core::graphic::polygon::default::Polygon;
use tearchan::core::graphic::shader::standard_2d_shader_program::{
    write_descriptor_sets, Standard2DShaderProgram,
};
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::extension::shared::Shared;
use tearchan::math::mesh::MeshBuilder;

pub struct HelloWorldScene {
    batch: Batch<Polygon, BatchBufferF32, Batch2D<BatchBufferF32>>,
    polygon: Shared<Polygon>,
    camera: Camera2D,
    shader_program: Standard2DShaderProgram,
    graphic_pipeline: FixedGraphicPipeline,
    texture: FixedTexture,
}

impl HelloWorldScene {
    pub fn creator() -> Option<SceneCreator> {
        Some(|scene_context| {
            let screen_size = scene_context.renderer_api.screen_size();
            let camera = Camera2D::new(screen_size.clone_owned());

            let image = Image::new_empty();
            let shader_program = Standard2DShaderProgram::new(scene_context.renderer_api, &camera);
            let texture = scene_context.renderer_api.create_texture(&image);
            let graphic_pipeline = scene_context
                .renderer_api
                .create_graphic_pipeline(shader_program.borrow_shader_program().borrow_shader());

            let mesh = MeshBuilder::new()
                .with_square(vec2(32.0f32, 32.0f32))
                .build()
                .unwrap();

            let mut batch = Batch2D::new(scene_context.renderer_api);
            let polygon = Shared::new(Polygon::new(mesh));
            batch.add(&polygon, 0);

            Box::new(HelloWorldScene {
                batch,
                polygon,
                camera,
                shader_program,
                graphic_pipeline,
                texture,
            })
        })
    }
}

impl SceneBase for HelloWorldScene {
    fn update(&mut self, scene_context: &mut SceneContext, _delta: f32) {
        self.camera.position.x += 0.1f32;
        /*{
            let mut polygon = self.polygon.borrow_mut();
            let next_position = &polygon.position + vec3(0.1f32, 0.0f32, 0.0f32);
            polygon.set_position(next_position);
        }*/
        self.camera.update();
        self.batch.flush();

        self.shader_program
            .prepare(self.camera.borrow_combine(), &self.texture);
        let descriptor_set = self.graphic_pipeline.borrow_descriptor_set();
        let write_descriptor_sets = write_descriptor_sets(
            descriptor_set,
            self.shader_program.borrow_mvp_matrix_uniform(),
            self.texture.borrow_image_view(),
            self.texture.borrow_sampler(),
        );
        scene_context
            .renderer_api
            .write_descriptor_sets(write_descriptor_sets);
        scene_context.renderer_api.draw_triangle(
            &self.graphic_pipeline,
            &self
                .batch
                .batch_buffers()
                .iter()
                .map(|x| x.borrow_vertex_buffer())
                .collect(),
            self.batch.triangle_count(),
        );
    }
}
