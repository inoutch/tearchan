use nalgebra_glm::{vec2, vec4, Mat4};
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::gfx::batch::batch2d::Batch2D;
use tearchan::gfx::batch::context::BatchContext;
use tearchan::gfx::batch::types::BatchTypeArray;
use tearchan::gfx::camera::Camera2D;
use tearchan::gfx::font_texture::FontTexture;
use tearchan::gfx::material::material2d::{Material2D, Material2DParams};
use tearchan::gfx::uniform_buffer::UniformBuffer;
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use tearchan::util::math::rect::rect2;
use tearchan::util::mesh::square::{
    create_square_colors, create_square_indices, create_square_positions, create_square_texcoords,
};
use tearchan::winit::event::WindowEvent;
use tearchan::winit::window::WindowBuilder;

#[allow(dead_code)]
struct FontScene {
    font_texture: FontTexture,
    batch: Batch2D,
    material: Material2D,
    transform_buffer: UniformBuffer<Mat4>,
    camera: Camera2D,
}

impl FontScene {
    pub fn factory() -> SceneFactory {
        |context, _| {
            let width = context.gfx().surface_config.width as f32;
            let height = context.gfx().surface_config.height as f32;
            let device = context.gfx().device;
            let queue = context.gfx().queue;

            // create font texture
            let sampler = device.create_sampler(&tearchan::gfx::wgpu::SamplerDescriptor {
                address_mode_u: tearchan::gfx::wgpu::AddressMode::ClampToEdge,
                address_mode_v: tearchan::gfx::wgpu::AddressMode::ClampToEdge,
                address_mode_w: tearchan::gfx::wgpu::AddressMode::ClampToEdge,
                mag_filter: tearchan::gfx::wgpu::FilterMode::Linear,
                min_filter: tearchan::gfx::wgpu::FilterMode::Linear,
                mipmap_filter: tearchan::gfx::wgpu::FilterMode::Nearest,
                ..Default::default()
            });
            let mut font_texture = FontTexture::new(
                device,
                include_bytes!("../assets/fonts/GenShinGothic-Medium.ttf").to_vec(),
                30.0f32,
                sampler,
                "FontTexture",
            )
            .unwrap();

            let mut batch = Batch2D::new(device);

            let square_idx = create_square_indices();
            let square_pos = create_square_positions(&rect2(80.0f32, 80.0f32, 256.0f32, 256.0f32));
            let square_tex = create_square_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32));
            let square_col = create_square_colors(vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32));
            batch.add(
                BatchTypeArray::V1U32 { data: square_idx },
                vec![
                    BatchTypeArray::V3F32 { data: square_pos },
                    BatchTypeArray::V2F32 { data: square_tex },
                    BatchTypeArray::V4F32 { data: square_col },
                ],
                None,
            );
            let text = r#"
            But I must explain to you how all this mistaken idea of denouncing pleasure
            and praising pain was born and I will give you a complete account of the
            system,
            "#;
            let (text_mesh, _text_size) = font_texture.create_mesh(device, queue, text).unwrap();
            batch.add(
                BatchTypeArray::V1U32 {
                    data: text_mesh.indices,
                },
                vec![
                    BatchTypeArray::V3F32 {
                        data: text_mesh.positions,
                    },
                    BatchTypeArray::V2F32 {
                        data: text_mesh.texcoords,
                    },
                    BatchTypeArray::V4F32 {
                        data: text_mesh.colors,
                    },
                ],
                None,
            );

            let mut camera = Camera2D::new(&vec2(width, height));
            camera.update();

            let transform_buffer = UniformBuffer::new(device, camera.combine());

            let material = Material2D::new(
                device,
                Material2DParams {
                    transform_buffer: transform_buffer.buffer(),
                    texture_view: &font_texture.view,
                    sampler: &font_texture.sampler,
                    color_format: context.gfx().surface_config.format,
                    shader_module: None,
                },
            );
            Box::new(FontScene {
                font_texture,
                batch,
                material,
                transform_buffer,
                camera,
            })
        }
    }
}

impl Scene for FontScene {
    fn update(&mut self, _context: &mut SceneContext, _event: WindowEvent) -> SceneControlFlow {
        SceneControlFlow::None
    }

    fn render(&mut self, context: &mut SceneRenderContext) -> SceneControlFlow {
        let queue = context.gfx().queue;
        let device = context.gfx().device;

        self.batch.flush(BatchContext { device, queue });

        let mut encoder = device
            .create_command_encoder(&tearchan::gfx::wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&tearchan::gfx::wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[tearchan::gfx::wgpu::RenderPassColorAttachment {
                    view: &context.gfx_rendering().view,
                    resolve_target: None,
                    ops: tearchan::gfx::wgpu::Operations {
                        load: tearchan::gfx::wgpu::LoadOp::Clear(tearchan::gfx::wgpu::Color {
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
            rpass.draw_indexed(0..self.batch.index_count() as u32, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        SceneControlFlow::None
    }
}

pub fn main() {
    env_logger::init();

    let window_builder = WindowBuilder::new().with_title("font");
    let startup_config = EngineStartupConfigBuilder::new()
        .window_builder(window_builder)
        .scene_factory(FontScene::factory())
        .build();
    let engine = Engine::new(startup_config);
    engine.run();
}
