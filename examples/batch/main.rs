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
use tearchan_gfx::batch::batch2d::Batch2D;
use tearchan_gfx::batch::types::{BatchTypeArray, BatchTypeTransform};
use tearchan_gfx::batch::{BatchCommandBuffer, BatchObjectId};
use tearchan_gfx::camera::Camera3D;
use tearchan_util::math::rect::rect2;
use tearchan_util::mesh::square::{
    create_square_colors, create_square_indices, create_square_positions, create_square_texcoords,
};
use wgpu::util::DeviceExt;
use winit::event::{ElementState, TouchPhase, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;

#[allow(dead_code)]
pub struct BatchScene {
    batch: Batch2D,
    batch_command_buffer: BatchCommandBuffer,
    sprites: VecDeque<BatchObjectId>,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    camera: Camera3D,
}

impl BatchScene {
    pub fn factory() -> SceneFactory {
        |context, _| {
            let gfx = context.gfx();
            let device = gfx.device;
            let queue = gfx.queue;
            let width = context.gfx().swapchain_desc.width as f32;
            let height = context.gfx().swapchain_desc.height as f32;
            let aspect = width / height;

            let mut sprites = VecDeque::new();
            let mut batch = Batch2D::new(device, queue);
            let mut batch_command_buffer = batch.create_command_buffer();
            create_sprite(&mut batch_command_buffer, &mut sprites);

            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStage::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(64),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                    ],
                });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
            let size = 1u32;
            let texel = vec![255, 255, 255, 255];
            let texture_extent = wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: texture_extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            });
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &texel,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(std::num::NonZeroU32::new(4 * size).unwrap()),
                    rows_per_image: None,
                },
                texture_extent,
            );
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });
            let mut camera = Camera3D::default_with_aspect(aspect);
            camera.position = vec3(0.0f32, 0.0f32, 4.0f32);
            camera.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            camera.up = vec3(0.0f32, 1.0f32, 0.0f32);
            camera.update();

            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(camera.combine().as_slice()),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

            // Create bind group
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: None,
            });

            // Create the render pipeline
            let vertex_state = [
                wgpu::VertexBufferLayout {
                    array_stride: 3 * std::mem::size_of::<f32>() as u64, // positions
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: 2 * std::mem::size_of::<f32>() as u64, // texcoords
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 1,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: 4 * std::mem::size_of::<f32>() as u64, // colors
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 2,
                    }],
                },
            ];

            let vs_module = device
                .create_shader_module(&wgpu::include_spirv!("../../target/shaders/batch.vert.spv"));
            let fs_module = device
                .create_shader_module(&wgpu::include_spirv!("../../target/shaders/batch.frag.spv"));

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &vertex_state,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs_module,
                    entry_point: "main",
                    targets: &[gfx.swapchain_desc.format.into()],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
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
                    let width = context.gfx().swapchain_desc.width as f64;
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
        let frame = context.gfx_rendering().frame();
        let queue = context.gfx().queue;
        let device = context.gfx().device;

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            self.batch.flush(device, queue, &mut Some(&mut encoder));
            let provider = self.batch.provider();

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
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
            rpass.set_index_buffer(provider.index_buffer().slice(..), wgpu::IndexFormat::Uint32);
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
    let window_builder = WindowBuilder::new().with_title("quad");
    let startup_config = EngineStartupConfigBuilder::new()
        .window_builder(window_builder)
        .scene_factory(BatchScene::factory())
        .build();
    let engine = Engine::new(startup_config);
    engine.run();
}
