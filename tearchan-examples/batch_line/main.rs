use nalgebra_glm::{vec3, vec4};
use rand::Rng;
use std::collections::VecDeque;
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::gfx::batch::types::BatchTypeArray;
use tearchan::gfx::batch::v2::batch_line::BatchLine;
use tearchan::gfx::batch::v2::context::BatchContext;
use tearchan::gfx::batch::v2::object_manager::BatchObjectId;
use tearchan::gfx::camera::Camera3D;
use tearchan::gfx::material::material_line::{MaterialLine, MaterialLineParams};
use tearchan::gfx::uniform_buffer::UniformBuffer;
use tearchan::gfx::wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor,
};
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;

struct BatchLineScene {
    camera: Camera3D,
    batch: BatchLine,
    material: MaterialLine,
    lines: VecDeque<BatchObjectId>,
}

impl BatchLineScene {
    pub fn factory() -> SceneFactory {
        |context, _| {
            let device = context.gfx().device;
            let aspect = context.gfx().surface_config.width as f32
                / context.gfx().surface_config.height as f32;

            let mut camera = Camera3D::new(aspect, 0.1f32, 10.0f32);
            camera.position = vec3(0.0f32, 0.0f32, 1.0f32);
            camera.update();

            let batch = BatchLine::new(device);

            let transform_buffer = UniformBuffer::new(device, camera.combine());

            let material = MaterialLine::new(
                device,
                MaterialLineParams {
                    transform_buffer: transform_buffer.buffer(),
                    color_format: context.gfx().surface_config.format,
                    shader_module: None,
                },
            );

            Box::new(BatchLineScene {
                camera,
                batch,
                material,
                lines: VecDeque::new(),
            })
        }
    }

    pub fn add_line(&mut self) {
        let mut rng = rand::thread_rng();
        let start = vec3(
            rng.gen_range(-1.0f32..1.0f32),
            rng.gen_range(-1.0f32..1.0f32),
            0.0f32,
        );
        let end = vec3(
            rng.gen_range(-1.0f32..1.0f32),
            rng.gen_range(-1.0f32..1.0f32),
            0.0f32,
        );
        let start_color = vec4(
            rng.gen_range(0.0f32..1.0f32),
            rng.gen_range(0.0f32..1.0f32),
            rng.gen_range(0.0f32..1.0f32),
            1.0f32,
        );
        let end_color = vec4(
            rng.gen_range(0.0f32..1.0f32),
            rng.gen_range(0.0f32..1.0f32),
            rng.gen_range(0.0f32..1.0f32),
            1.0f32,
        );

        let id = self.batch.add(
            BatchTypeArray::V1U32 { data: vec![0, 1] },
            vec![
                BatchTypeArray::V3F32 {
                    data: vec![start, end],
                },
                BatchTypeArray::V4F32 {
                    data: vec![start_color, end_color],
                },
            ],
            None,
        );
        self.lines.push_back(id);
    }

    pub fn remove_line(&mut self) {
        if let Some(id) = self.lines.pop_front() {
            self.batch.remove(id);
        }
    }
}

impl Scene for BatchLineScene {
    fn update(&mut self, _context: &mut SceneContext, event: WindowEvent) -> SceneControlFlow {
        match event {
            WindowEvent::Resized(size) => {
                let width = size.width.max(1);
                let height = size.height.max(1);
                let aspect = width as f32 / height as f32;
                self.camera = Camera3D::default_with_aspect(aspect);
                self.camera.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
                self.camera.up = vec3(0.0f32, 1.0f32, 0.0f32);
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    if input.state == ElementState::Pressed {
                        match key {
                            VirtualKeyCode::Right => self.add_line(),
                            VirtualKeyCode::Left => self.remove_line(),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        SceneControlFlow::None
    }

    fn render(&mut self, context: &mut SceneRenderContext) -> SceneControlFlow {
        let queue = context.gfx().queue;
        let device = context.gfx().device;

        self.batch.flush(BatchContext { device, queue });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let index_count = self.batch.index_count() as u32;
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &context.gfx_rendering().view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.material.bind(&mut rpass);
            self.batch.bind(&mut rpass);
            rpass.draw_indexed(0..index_count, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        SceneControlFlow::None
    }
}

fn main() {
    env_logger::init();

    let window_builder = WindowBuilder::new().with_title("Line example");
    let startup_config = EngineStartupConfigBuilder::new()
        .window_builder(window_builder)
        .scene_factory(BatchLineScene::factory())
        .build();
    let engine = Engine::new(startup_config);
    engine.run();
}
