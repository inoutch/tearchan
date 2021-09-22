use color::{Deg, Hsv, Rgb, ToRgb};
use nalgebra_glm::{rotate_z, vec3, vec4, Mat4};
use rand::Rng;
use std::collections::VecDeque;
use std::f32::consts::PI;
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use tearchan::gfx::batch::batch2d::Batch2D;
use tearchan::gfx::batch::types::{BatchTypeArray, BatchTypeTransform};
use tearchan::gfx::batch::{BatchCommandBuffer, BatchObjectId};
use tearchan::gfx::camera::Camera3D;
use tearchan::util::math::rect::rect2;
use tearchan::util::mesh::square::{
    create_square_colors, create_square_indices, create_square_positions, create_square_texcoords,
};
use tearchan::gfx::wgpu::util::DeviceExt;
use winit::event::{ElementState, TouchPhase, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;
use tearchan::gfx::wgpu::TextureAspect;

#[allow(dead_code)]
pub struct BatchScene {
    batch: Batch2D,
    batch_command_buffer: BatchCommandBuffer,
    sprites: VecDeque<BatchObjectId>,
    bind_group: tearchan::gfx::wgpu::BindGroup,
    uniform_buffer: tearchan::gfx::wgpu::Buffer,
    pipeline: tearchan::gfx::wgpu::RenderPipeline,
    camera: Camera3D,
}

impl BatchScene {
    pub fn factory() -> SceneFactory {
        |context, _| {
            let gfx = context.gfx();
            let device = gfx.device;
            let queue = gfx.queue;
            let width = context.gfx().surface_config.width as f32;
            let height = context.gfx().surface_config.height as f32;
            let aspect = width / height;

            let mut sprites = VecDeque::new();
            let mut batch = Batch2D::new(device, queue);
            let mut batch_command_buffer = batch.create_command_buffer();
            create_sprite(&mut batch_command_buffer, &mut sprites);

            let bind_group_layout =
                device.create_bind_group_layout(&tearchan::gfx::wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        tearchan::gfx::wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: tearchan::gfx::wgpu::ShaderStages::VERTEX,
                            ty: tearchan::gfx::wgpu::BindingType::Buffer {
                                ty: tearchan::gfx::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: tearchan::gfx::wgpu::BufferSize::new(64),
                            },
                            count: None,
                        },
                        tearchan::gfx::wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: tearchan::gfx::wgpu::ShaderStages::FRAGMENT,
                            ty: tearchan::gfx::wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: tearchan::gfx::wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: tearchan::gfx::wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        tearchan::gfx::wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: tearchan::gfx::wgpu::ShaderStages::FRAGMENT,
                            ty: tearchan::gfx::wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                    ],
                });
            let pipeline_layout = device.create_pipeline_layout(&tearchan::gfx::wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
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
                usage: tearchan::gfx::wgpu::TextureUsages::TEXTURE_BINDING | tearchan::gfx::wgpu::TextureUsages::RENDER_ATTACHMENT | tearchan::gfx::wgpu::TextureUsages::COPY_DST,
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

            let uniform_buffer = device.create_buffer_init(&tearchan::gfx::wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(camera.combine().as_slice()),
                usage: tearchan::gfx::wgpu::BufferUsages::UNIFORM | tearchan::gfx::wgpu::BufferUsages::COPY_DST,
            });

            // Create bind group
            let bind_group = device.create_bind_group(&tearchan::gfx::wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 1,
                        resource: tearchan::gfx::wgpu::BindingResource::TextureView(&texture_view),
                    },
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 2,
                        resource: tearchan::gfx::wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: None,
            });

            // Create the render pipeline
            let vertex_state = [
                tearchan::gfx::wgpu::VertexBufferLayout {
                    array_stride: 3 * std::mem::size_of::<f32>() as u64, // positions
                    step_mode: tearchan::gfx::wgpu::VertexStepMode::Vertex,
                    attributes: &[tearchan::gfx::wgpu::VertexAttribute {
                        format: tearchan::gfx::wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                },
                tearchan::gfx::wgpu::VertexBufferLayout {
                    array_stride: 2 * std::mem::size_of::<f32>() as u64, // texcoords
                    step_mode: tearchan::gfx::wgpu::VertexStepMode::Vertex,
                    attributes: &[tearchan::gfx::wgpu::VertexAttribute {
                        format: tearchan::gfx::wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 1,
                    }],
                },
                tearchan::gfx::wgpu::VertexBufferLayout {
                    array_stride: 4 * std::mem::size_of::<f32>() as u64, // colors
                    step_mode: tearchan::gfx::wgpu::VertexStepMode::Vertex,
                    attributes: &[tearchan::gfx::wgpu::VertexAttribute {
                        format: tearchan::gfx::wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 2,
                    }],
                },
            ];

            let shader = device.create_shader_module(&tearchan::gfx::wgpu::include_wgsl!("./batch.wgsl"));

            let pipeline = device.create_render_pipeline(&tearchan::gfx::wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: tearchan::gfx::wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &vertex_state,
                },
                fragment: Some(tearchan::gfx::wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[gfx.surface_config.format.into()],
                }),
                primitive: tearchan::gfx::wgpu::PrimitiveState {
                    cull_mode: Some(tearchan::gfx::wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: tearchan::gfx::wgpu::MultisampleState::default(),
            });

            Box::new(BatchScene {
                batch,
                batch_command_buffer,
                sprites,
                bind_group,
                uniform_buffer,
                pipeline,
                camera,
            })
        }
    }
}

impl Scene for BatchScene {
    fn update(&mut self, context: &mut SceneContext, event: WindowEvent) -> SceneControlFlow {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    if input.state == ElementState::Pressed {
                        match key {
                            VirtualKeyCode::Right => {
                                create_sprite(&mut self.batch_command_buffer, &mut self.sprites);
                            }
                            VirtualKeyCode::Left => {
                                destroy_sprite(&mut self.batch_command_buffer, &mut self.sprites);
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::Touch(touch) => match touch.phase {
                TouchPhase::Started => {
                    let width = context.gfx().surface_config.width as f64;
                    if width / 2.0f64 > touch.location.x {
                        create_sprite(&mut self.batch_command_buffer, &mut self.sprites);
                    } else {
                        destroy_sprite(&mut self.batch_command_buffer, &mut self.sprites);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        SceneControlFlow::None
    }

    fn render(&mut self, context: &mut SceneRenderContext) -> SceneControlFlow {
        let queue = context.gfx().queue;
        let device = context.gfx().device;

        let mut encoder =
            device.create_command_encoder(&tearchan::gfx::wgpu::CommandEncoderDescriptor { label: None });
        {
            self.batch.flush(device, queue, &mut Some(&mut encoder));
            let provider = self.batch.provider();

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
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(provider.index_buffer().slice(..), tearchan::gfx::wgpu::IndexFormat::Uint32);
            rpass.set_vertex_buffer(0, provider.position_buffer().slice(..));
            rpass.set_vertex_buffer(1, provider.texcoord_buffer().slice(..));
            rpass.set_vertex_buffer(2, provider.color_buffer().slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..provider.index_count() as u32, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        SceneControlFlow::None
    }
}

pub fn create_sprite(
    command_buffer: &mut BatchCommandBuffer,
    sprites: &mut VecDeque<BatchObjectId>,
) {
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
    let id = command_buffer.add(
        vec![
            BatchTypeArray::V1U32 {
                data: square_indices,
            },
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
    command_buffer.transform(
        id,
        1,
        BatchTypeTransform::Mat4F32 {
            m: rotate_z(&Mat4::identity(), rng.gen_range(0.0f32..PI * 2.0f32)),
        },
    );
    sprites.push_back(id);
}

pub fn destroy_sprite(
    command_buffer: &mut BatchCommandBuffer,
    sprites: &mut VecDeque<BatchObjectId>,
) {
    let id = match sprites.pop_front() {
        None => return,
        Some(id) => id,
    };
    command_buffer.remove(id);
}

pub fn main() {
    env_logger::init();

    let window_builder = WindowBuilder::new().with_title("empty");
    let startup_config = EngineStartupConfigBuilder::new()
        .window_builder(window_builder)
        .scene_factory(BatchScene::factory())
        .build();
    let engine = Engine::new(startup_config);
    engine.run();
}
