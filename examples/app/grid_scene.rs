use gfx_hal::pso::{FrontFace, PolygonMode, Primitive, Rasterizer, State};
use nalgebra_glm::vec3;
use std::ops::Range;
use tearchan::core::graphic::batch::batch_line::BatchLine;
use tearchan::core::graphic::camera_3d::Camera3D;
use tearchan::core::graphic::hal::backend::GraphicPipeline;
use tearchan::core::graphic::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan::core::graphic::hal::renderer::ResizeContext;
use tearchan::core::graphic::polygon::{Polygon, PolygonCommon};
use tearchan::core::graphic::shader::grid_shader_program::GridShaderProgram;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::core::scene::touch::Touch;
use tearchan::extension::shared::make_shared;
use tearchan::math::mesh::MeshBuilder;
use winit::event::KeyboardInput;

pub struct GridScene {
    camera: Camera3D,
    batch: BatchLine,
    shader_program: GridShaderProgram,
    graphic_pipeline: GraphicPipeline,
}

impl GridScene {
    pub fn creator() -> SceneCreator {
        |context, _| {
            let screen_size = &context.renderer_api.display_size().screen;

            let mut camera = Camera3D::default_with_aspect(screen_size.x / screen_size.y);
            camera.position = vec3(0.0f32, 2.0f32, 4.0f32);
            camera.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            camera.up = vec3(0.0f32, 1.0f32, 0.0f32);
            camera.update();

            let shader_program = GridShaderProgram::new(context.renderer_api, camera.base());
            let graphic_pipeline = context.renderer_api.create_graphic_pipeline(
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

            let mut batch = BatchLine::new_batch_line(context.renderer_api);
            let mesh = MeshBuilder::new()
                .with_grid(
                    0.5f32,
                    Range {
                        start: (-5, -5),
                        end: (5, 5),
                    },
                )
                .build()
                .unwrap();
            let polygon = make_shared(Polygon::new(mesh));
            polygon
                .borrow_mut()
                .set_rotation_radian(std::f32::consts::PI * 0.5f32);
            polygon
                .borrow_mut()
                .set_rotation_axis(vec3(1.0f32, 0.0f32, 0.0f32));
            batch.add(&polygon, 0);

            Box::new(GridScene {
                camera,
                batch,
                shader_program,
                graphic_pipeline,
            })
        }
    }
}

impl SceneBase for GridScene {
    fn update(&mut self, context: &mut SceneContext, _delta: f32) {
        self.camera.update();
        self.batch.flush();

        self.shader_program.prepare(self.camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        let write_descriptor_sets = self
            .shader_program
            .create_write_descriptor_sets(descriptor_set);

        context
            .renderer_api
            .write_descriptor_sets(write_descriptor_sets);
        context.renderer_api.draw_elements(
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
