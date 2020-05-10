use nalgebra_glm::{vec2, vec3, vec4};
use tearchan::core::graphic::batch::batch2d::Batch2D;
use tearchan::core::graphic::batch::batch3d::Batch3D;
use tearchan::core::graphic::batch::batch_buffer_f32::BatchBufferF32;
use tearchan::core::graphic::batch::default::Batch;
use tearchan::core::graphic::camera_2d::Camera2D;
use tearchan::core::graphic::camera_3d::Camera3D;
use tearchan::core::graphic::hal::backend::{FixedGraphicPipeline, FixedTexture};
use tearchan::core::graphic::hal::image::Image;
use tearchan::core::graphic::polygon::default::Polygon;
use tearchan::core::graphic::shader::standard_2d_shader_program::{
    write_descriptor_sets, Standard2DShaderProgram,
};
use tearchan::core::graphic::shader::standard_3d_shader_program::Standard3DShaderProgram;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::extension::shared::Shared;
use tearchan::math::mesh::MeshBuilder;

pub struct HelloWorldScene {
    camera: Camera2D,
    batch: Batch<Polygon, BatchBufferF32, Batch2D<BatchBufferF32>>,
    polygon: Shared<Polygon>,
    shader_program: Standard2DShaderProgram,
    graphic_pipeline: FixedGraphicPipeline,
    batch_3d: Batch<Polygon, BatchBufferF32, Batch3D<BatchBufferF32>>,
    polygon_2d: Shared<Polygon>,
    camera_3d: Camera3D,
    shader_program_3d: Standard3DShaderProgram,
    graphic_pipeline_3d: FixedGraphicPipeline,
    texture: FixedTexture,
}

impl HelloWorldScene {
    pub fn creator() -> Option<SceneCreator> {
        Some(|scene_context| {
            let screen_size = scene_context.renderer_api.screen_size();
            let image = Image::new_empty();

            let camera = Camera2D::new(screen_size.clone_owned());
            let mut camera_3d = Camera3D::new(screen_size.x / screen_size.y);
            camera_3d.position = vec3(0.0f32, 2.0f32, -4.0f32);
            camera_3d.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            camera_3d.up = vec3(0.0f32, 1.0f32, 0.0f32);
            camera_3d.update();

            //
            let texture = scene_context.renderer_api.create_texture(&image);

            let shader_program = Standard2DShaderProgram::new(scene_context.renderer_api, camera.borrow_base());
            let graphic_pipeline = scene_context
                .renderer_api
                .create_graphic_pipeline(shader_program.borrow_shader_program().borrow_shader());

            let standard_3d_shader_program =
                Standard3DShaderProgram::new(scene_context.renderer_api, camera_3d.borrow_base());
            let graphic_pipeline_3d = scene_context.renderer_api.create_graphic_pipeline(
                standard_3d_shader_program
                    .borrow_shader_program()
                    .borrow_shader(),
            );

            let mesh = MeshBuilder::new()
                .with_square(vec2(320.0f32, 640.0f32))
                .build()
                .unwrap();
            let mut batch = Batch2D::new(scene_context.renderer_api);
            let polygon = Shared::new(Polygon::new(mesh));
            // batch.add(&polygon, 0);

            let mesh3 = MeshBuilder::new().with_cube(2.0f32).build().unwrap();
            let mut batch_3d = Batch3D::new(scene_context.renderer_api);
            let polygon_2d = Shared::new(Polygon::new(mesh3));
            batch_3d.add(&polygon_2d, 0);

            Box::new(HelloWorldScene {
                batch,
                polygon,
                camera,
                shader_program,
                graphic_pipeline,
                camera_3d,
                batch_3d,
                polygon_2d,
                shader_program_3d: standard_3d_shader_program,
                graphic_pipeline_3d,
                texture,
            })
        })
    }
}

impl SceneBase for HelloWorldScene {
    fn update(&mut self, scene_context: &mut SceneContext, _delta: f32) {
        // self.camera.position.x += 0.1f32;
        /*self.camera.update();
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
        );*/
        self.camera_3d.update();
        self.batch_3d.flush();

        self.shader_program_3d.prepare(
            self.camera_3d.borrow_combine(),
            &vec3(0.0f32, 0.0f32, -1.0f32),
            &vec3(1.0f32, 1.0f32, 1.0f32),
            1.0f32,
            &self.texture,
        );

        let descriptor_set = self.graphic_pipeline_3d.borrow_descriptor_set();
        let write_descriptor_sets =
            tearchan::core::graphic::shader::standard_3d_shader_program::write_descriptor_sets(
                descriptor_set,
                &self.shader_program_3d.vp_matrix_uniform,
                /*&self.shader_program_3d.inv_vp_matrix_uniform,
                &self.shader_program_3d.light_position_uniform,
                &self.shader_program_3d.light_color_uniform,
                &self.shader_program_3d.ambient_strength_uniform,*/
                self.texture.borrow_image_view(),
                self.texture.borrow_sampler(),
            );

        scene_context
            .renderer_api
            .write_descriptor_sets(write_descriptor_sets);
        scene_context.renderer_api.draw_triangle(
            &self.graphic_pipeline_3d,
            &self
                .batch_3d
                .batch_buffers()
                .iter()
                .map(|x| x.borrow_vertex_buffer())
                .collect(),
            self.batch_3d.triangle_count(),
        );
    }
}
