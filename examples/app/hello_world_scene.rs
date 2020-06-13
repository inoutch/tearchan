use nalgebra_glm::vec3;
use tearchan::core::graphic::batch::batch3d::Batch3D;
use tearchan::core::graphic::batch::batch_buffer_f32::BatchBufferF32;
use tearchan::core::graphic::batch::Batch;
use tearchan::core::graphic::camera_3d::Camera3D;
use tearchan::core::graphic::hal::backend::{GraphicPipeline, Texture};
use tearchan::core::graphic::hal::texture::TextureConfig;
use tearchan::core::graphic::image::Image;
use tearchan::core::graphic::polygon::{Polygon, PolygonCommon};
use tearchan::core::graphic::shader::standard_3d_shader_program::Standard3DShaderProgram;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::extension::shared::{make_shared, Shared};
use tearchan::math::mesh::MeshBuilder;

pub struct HelloWorldScene {
    camera: Camera3D,
    batch: Batch<Polygon, BatchBufferF32, Batch3D<BatchBufferF32>>,
    shader_program: Standard3DShaderProgram,
    texture: Texture,
    graphic_pipeline: GraphicPipeline,
    polygon: Shared<Polygon>,
}

impl HelloWorldScene {
    pub fn creator() -> SceneCreator {
        |scene_context, _| {
            let screen_size = scene_context.renderer_api.screen_size();
            let image = Image::new_empty();

            let mut camera_3d = Camera3D::default_with_aspect(screen_size.x / screen_size.y);
            camera_3d.position = vec3(0.0f32, -2.0f32, 4.0f32);
            camera_3d.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            camera_3d.up = vec3(0.0f32, 1.0f32, 0.0f32);
            camera_3d.update();

            //
            let texture = scene_context
                .renderer_api
                .create_texture(&image, TextureConfig::default());

            let standard_3d_shader_program =
                Standard3DShaderProgram::new(scene_context.renderer_api, camera_3d.base());
            let graphic_pipeline_3d = scene_context
                .renderer_api
                .create_graphic_pipeline(standard_3d_shader_program.shader());

            let mesh3 = MeshBuilder::new().with_cube(1.0f32).build().unwrap();
            let mut batch_3d = Batch3D::new(scene_context.renderer_api);
            let polygon_2d = make_shared(Polygon::new(mesh3));
            polygon_2d
                .borrow_mut()
                .set_rotation_axis(vec3(0.0f32, 1.0f32, 1.0f32));
            batch_3d.add(&polygon_2d, 0);

            Box::new(HelloWorldScene {
                camera: camera_3d,
                batch: batch_3d,
                polygon: polygon_2d,
                shader_program: standard_3d_shader_program,
                graphic_pipeline: graphic_pipeline_3d,
                texture,
            })
        }
    }
}

impl SceneBase for HelloWorldScene {
    fn update(&mut self, scene_context: &mut SceneContext, _delta: f32) {
        let rotation = self.polygon.borrow().rotation_radian() + 0.01f32;
        self.polygon.borrow_mut().set_rotation_radian(rotation);
        self.camera.update();
        self.batch.flush();

        self.shader_program.prepare(
            self.camera.combine(),
            &vec3(0.0f32, -2.0f32, 4.0f32),
            &vec3(1.0f32, 1.0f32, 1.0f32),
            0.2f32,
        );

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        let write_descriptor_sets = self
            .shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture);

        scene_context
            .renderer_api
            .write_descriptor_sets(write_descriptor_sets);
        scene_context.renderer_api.draw_triangle(
            &self.graphic_pipeline,
            &self
                .batch
                .batch_buffers()
                .iter()
                .map(|x| x.vertex_buffer())
                .collect::<Vec<_>>(),
            self.batch.triangle_count(),
        );
    }
}
