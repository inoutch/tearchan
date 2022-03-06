use color::{Deg, Hsv, Rgb, ToRgb};
use nalgebra_glm::{rotate_z, vec3, vec4, Mat4};
use rand::Rng;
use std::collections::VecDeque;
use std::f32::consts::PI;
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::gfx::batch::batch2d::Batch2D;
use tearchan::gfx::batch::batch2d::BATCH2D_ATTRIBUTE_POSITION;
use tearchan::gfx::batch::context::BatchContext;
use tearchan::gfx::batch::object_manager::BatchObjectId;
use tearchan::gfx::batch::types::{BatchTypeArray, BatchTypeTransform};
use tearchan::gfx::camera::Camera3D;
use tearchan::gfx::material::material2d::{Material2D, Material2DParams};
use tearchan::gfx::uniform_buffer::UniformBuffer;
use tearchan::gfx::wgpu::{Device, Queue, TextureAspect, TextureView};
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use tearchan::util::math::rect::rect2;
use tearchan::util::mesh::square::{
    create_square_colors, create_square_indices, create_square_positions, create_square_texcoords,
};
use winit::event::{ElementState, TouchPhase, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;

#[allow(dead_code)]
struct Batch2DScene {
    batch: Batch2D,
    material: Material2D,
    sprites: VecDeque<BatchObjectId>,
    transform_buffer: UniformBuffer<Mat4>,
    camera: Camera3D,
}

impl Batch2DScene {
    pub fn factory() -> SceneFactory {
        |context, _| {
            let gfx = context.gfx();
            let device = gfx.device;
            let queue = gfx.queue;
            let width = context.gfx().surface_config.width as f32;
            let height = context.gfx().surface_config.height as f32;
            let aspect = width / height;

            let mut sprites = VecDeque::new();
            let mut batch = Batch2D::new(device);
            create_sprite(&mut batch, &mut sprites);

            let texture_view = create_texture_view(device, queue);
            let sampler = device.create_sampler(&tearchan::gfx::wgpu::SamplerDescriptor {
                address_mode_u: tearchan::gfx::wgpu::AddressMode::ClampToEdge,
                address_mode_v: tearchan::gfx::wgpu::AddressMode::ClampToEdge,
                address_mode_w: tearchan::gfx::wgpu::AddressMode::ClampToEdge,
                mag_filter: tearchan::gfx::wgpu::FilterMode::Nearest,
                min_filter: tearchan::gfx::wgpu::FilterMode::Linear,
                mipmap_filter: tearchan::gfx::wgpu::FilterMode::Nearest,
                ..Default::default()
            });
            let mut camera = Camera3D::default_with_aspect(aspect);
            camera.position = vec3(0.0f32, 0.0f32, 4.0f32);
            camera.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            camera.up = vec3(0.0f32, 1.0f32, 0.0f32);
            camera.update();

            let transform_buffer = UniformBuffer::new(device, camera.combine());

            let shader_module =
                device.create_shader_module(&tearchan::gfx::wgpu::include_wgsl!("./batch.wgsl"));
            let material = Material2D::new(
                device,
                Material2DParams {
                    transform_buffer: transform_buffer.buffer(),
                    texture_view: &texture_view,
                    sampler: &sampler,
                    color_format: context.gfx().surface_config.format,
                    shader_module: Some(shader_module),
                },
            );

            Box::new(Batch2DScene {
                batch,
                material,
                sprites,
                transform_buffer,
                camera,
            })
        }
    }
}

impl Scene for Batch2DScene {
    fn update(&mut self, context: &mut SceneContext, event: WindowEvent) -> SceneControlFlow {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    if input.state == ElementState::Pressed {
                        match key {
                            VirtualKeyCode::Right => {
                                create_sprite(&mut self.batch, &mut self.sprites);
                            }
                            VirtualKeyCode::Left => {
                                destroy_sprite(&mut self.batch, &mut self.sprites);
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::Touch(touch) => {
                if touch.phase == TouchPhase::Started {
                    let width = context.gfx().surface_config.width as f64;
                    if width / 2.0f64 > touch.location.x {
                        create_sprite(&mut self.batch, &mut self.sprites);
                    } else {
                        destroy_sprite(&mut self.batch, &mut self.sprites);
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

        let mut encoder = device
            .create_command_encoder(&tearchan::gfx::wgpu::CommandEncoderDescriptor { label: None });
        {
            let index_count = self.batch.index_count() as u32;
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
            rpass.push_debug_group("Prepare data for draw.");
            self.material.bind(&mut rpass);
            self.batch.bind(&mut rpass);
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..index_count, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        SceneControlFlow::None
    }
}

fn create_sprite(batch: &mut Batch2D, sprites: &mut VecDeque<BatchObjectId>) {
    let mut rng = rand::thread_rng();
    let color: Rgb<f32> = Hsv::new(Deg(rng.gen_range(0.0f32..360.0f32)), 1.0f32, 1.0f32).to_rgb();
    let square_indices = create_square_indices();
    let square_positions = create_square_positions(&rect2(
        rng.gen_range(-1.0f32..1.0f32),
        rng.gen_range(-1.0f32..1.0f32),
        1.0f32,
        1.0f32,
    ));
    let square_colors = create_square_colors(vec4(color.r, color.g, color.b, 1.0f32));
    let square_texcoords = create_square_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32));
    let id = batch.add(
        BatchTypeArray::V1U32 {
            data: square_indices,
        },
        vec![
            BatchTypeArray::V3F32 {
                data: square_positions,
            },
            BatchTypeArray::V2F32 {
                data: square_texcoords,
            },
            BatchTypeArray::V4F32 {
                data: square_colors,
            },
        ],
        None,
    );
    batch.transform(
        id,
        BATCH2D_ATTRIBUTE_POSITION,
        BatchTypeTransform::Mat4F32 {
            m: rotate_z(&Mat4::identity(), rng.gen_range(0.0f32..PI * 2.0f32)),
        },
    );
    sprites.push_back(id);
}

fn destroy_sprite(batch: &mut Batch2D, sprites: &mut VecDeque<BatchObjectId>) {
    let id = match sprites.pop_front() {
        None => return,
        Some(id) => id,
    };
    batch.remove(id);
}

fn create_texture_view(device: &Device, queue: &Queue) -> TextureView {
    let size = 1u32;
    let texel = vec![255, 255, 255, 255];
    let texture_extent = tearchan::gfx::wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&tearchan::gfx::wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: tearchan::gfx::wgpu::TextureDimension::D2,
        format: tearchan::gfx::wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: tearchan::gfx::wgpu::TextureUsages::TEXTURE_BINDING
            | tearchan::gfx::wgpu::TextureUsages::RENDER_ATTACHMENT
            | tearchan::gfx::wgpu::TextureUsages::COPY_DST,
    });
    let texture_view = texture.create_view(&tearchan::gfx::wgpu::TextureViewDescriptor::default());
    queue.write_texture(
        tearchan::gfx::wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: tearchan::gfx::wgpu::Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &texel,
        tearchan::gfx::wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(std::num::NonZeroU32::new(4 * size).unwrap()),
            rows_per_image: None,
        },
        texture_extent,
    );
    texture_view
}

fn main() {
    env_logger::init();

    let window_builder = WindowBuilder::new().with_title("Batch2D example");
    let startup_config = EngineStartupConfigBuilder::new()
        .window_builder(window_builder)
        .scene_factory(Batch2DScene::factory())
        .build();
    let engine = Engine::new(startup_config);
    engine.run();
}
