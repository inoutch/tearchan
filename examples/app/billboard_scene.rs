use crate::texture_bundle::generate_texture_bundle;
use gfx_hal::pso::{FrontFace, PolygonMode, Primitive, Rasterizer, State};
use nalgebra_glm::{vec2, vec3};
use std::ops::Range;
use tearchan::core::graphic::batch::batch3d::Batch3D;
use tearchan::core::graphic::batch::batch_buffer_f32::BatchBufferF32;
use tearchan::core::graphic::batch::batch_line::BatchLine;
use tearchan::core::graphic::batch::Batch;
use tearchan::core::graphic::camera_3d::Camera3D;
use tearchan::core::graphic::hal::backend::{GraphicPipeline, Texture};
use tearchan::core::graphic::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan::core::graphic::hal::texture::TextureConfig;
use tearchan::core::graphic::polygon::{Polygon, PolygonCommon};
use tearchan::core::graphic::shader::grid_shader_program::GridShaderProgram;
use tearchan::core::graphic::shader::standard_3d_shader_program::Standard3DShaderProgram;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::core::scene::touch::Touch;
use tearchan::extension::shared::make_shared;
use tearchan::math::mesh::MeshBuilder;
use winit::event::KeyboardInput;

pub struct BillboardScene {
    camera: Camera3D,
    grid_batch: Batch<Polygon, BatchBufferF32, BatchLine<BatchBufferF32>>,
    grid_shader_program: GridShaderProgram,
    grid_graphic_pipeline: GraphicPipeline,
    billboard_batch: Batch<Polygon, BatchBufferF32, Batch3D<BatchBufferF32>>,
    billboard_shader_program: Standard3DShaderProgram,
    billboard_graphic_pipeline: GraphicPipeline,
    billboard_texture: Texture,
}

impl BillboardScene {
    pub fn creator() -> SceneCreator {
        |ctx, _| Box::new(BillboardScene::new(ctx))
    }

    pub fn new(ctx: &mut SceneContext) -> Self {
        let screen_size = ctx.renderer_api.screen_size();

        let mut camera = Camera3D::default_with_aspect(screen_size.x / screen_size.y);
        camera.position = vec3(0.0f32, 2.0f32, 4.0f32);
        camera.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
        camera.up = vec3(0.0f32, 1.0f32, 0.0f32);
        camera.update();

        let grid_shader_program = GridShaderProgram::new(ctx.renderer_api, camera.base());
        let grid_graphic_pipeline = ctx.renderer_api.create_graphic_pipeline(
            grid_shader_program.shader(),
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

        let mut grid_batch = BatchLine::new(ctx.renderer_api);
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
        grid_batch.add(&polygon, 0);

        let (texture_atlas, image) = generate_texture_bundle();
        let point_sprite_texture = ctx
            .renderer_api
            .create_texture(&image, TextureConfig::default());

        let mut point_sprite_batch = Batch3D::new(ctx.renderer_api);
        let point_sprite_shader_program =
            Standard3DShaderProgram::new(ctx.renderer_api, camera.base());
        let point_sprite_graphic_pipeline = ctx.renderer_api.create_graphic_pipeline(
            point_sprite_shader_program.shader(),
            GraphicPipelineConfig::default(),
        );

        {
            let frame = &texture_atlas.frames[0];
            let mesh = MeshBuilder::new()
                .with_frame(
                    vec2(
                        point_sprite_texture.size().x as f32,
                        point_sprite_texture.size().y as f32,
                    ),
                    frame,
                )
                .build()
                .unwrap();
            let polygon = make_shared(Polygon::new(mesh));
            polygon
                .borrow_mut()
                .set_scale(vec3(0.005f32, 0.005f32, 0.005f32));
            point_sprite_batch.add(&polygon, 0);
        }

        BillboardScene {
            camera,
            grid_batch,
            grid_shader_program,
            grid_graphic_pipeline,
            billboard_batch: point_sprite_batch,
            billboard_shader_program: point_sprite_shader_program,
            billboard_graphic_pipeline: point_sprite_graphic_pipeline,
            billboard_texture: point_sprite_texture,
        }
    }
}

impl SceneBase for BillboardScene {
    fn update(&mut self, ctx: &mut SceneContext, _delta: f32) {
        self.camera.update();
        self.grid_batch.flush();
        self.billboard_batch.flush();

        self.grid_shader_program.prepare(self.camera.combine());

        let descriptor_set = self.grid_graphic_pipeline.descriptor_set();
        let write_descriptor_sets = self
            .grid_shader_program
            .create_write_descriptor_sets(descriptor_set);

        ctx.renderer_api
            .write_descriptor_sets(write_descriptor_sets);
        ctx.renderer_api.draw_vertices(
            &self.grid_graphic_pipeline,
            &self
                .grid_batch
                .batch_buffers()
                .iter()
                .map(|x| x.vertex_buffer())
                .collect::<Vec<_>>(),
            self.grid_batch.vertex_count(),
        );

        self.billboard_shader_program.prepare(
            self.camera.combine(),
            &vec3(0.0f32, 2.0f32, 4.0f32),
            &vec3(1.0f32, 1.0f32, 1.0f32),
            0.2f32,
        );

        let descriptor_set = self.billboard_graphic_pipeline.descriptor_set();
        let write_descriptor_sets = self
            .billboard_shader_program
            .create_write_descriptor_sets(descriptor_set, &self.billboard_texture);

        ctx.renderer_api
            .write_descriptor_sets(write_descriptor_sets);
        ctx.renderer_api.draw_vertices(
            &self.billboard_graphic_pipeline,
            &self
                .billboard_batch
                .batch_buffers()
                .iter()
                .map(|x| x.vertex_buffer())
                .collect::<Vec<_>>(),
            self.billboard_batch.vertex_count(),
        );
    }

    fn on_touch_start(&mut self, _touch: &Touch) {}

    fn on_touch_end(&mut self, _touch: &Touch) {}

    fn on_touch_move(&mut self, _touch: &Touch) {}

    fn on_touch_cancel(&mut self, _touch: &Touch) {}

    fn on_key_down(&mut self, _input: &KeyboardInput) {}

    fn on_key_up(&mut self, _input: &KeyboardInput) {}
}
