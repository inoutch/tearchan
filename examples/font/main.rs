use nalgebra_glm::{vec2, vec4};
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::gfx::batch::batch2d::Batch2D;
use tearchan::gfx::batch::types::BatchTypeArray;
use tearchan::gfx::camera::Camera2D;
use tearchan::gfx::font_texture::FontTexture;
use tearchan::gfx::shader::get_standard_2d_shader_source;
use tearchan::gfx::wgpu::util::DeviceExt;
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
    bind_group: tearchan::gfx::wgpu::BindGroup,
    bind_group_layout: tearchan::gfx::wgpu::BindGroupLayout,
    uniform_buffer: tearchan::gfx::wgpu::Buffer,
    pipeline: tearchan::gfx::wgpu::RenderPipeline,
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

            let mut batch = Batch2D::new(device, queue);
            let mut batch_command_buffer = batch.create_command_buffer();

            let square_idx = create_square_indices();
            let square_pos = create_square_positions(&rect2(80.0f32, 80.0f32, 256.0f32, 256.0f32));
            let square_tex = create_square_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32));
            let square_col = create_square_colors(vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32));
            batch_command_buffer.add(
                vec![
                    BatchTypeArray::V1U32 { data: square_idx },
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
            batch_command_buffer.add(
                vec![
                    BatchTypeArray::V1U32 {
                        data: text_mesh.indices,
                    },
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
                                sample_type: tearchan::gfx::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
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
            let pipeline_layout =
                device.create_pipeline_layout(&tearchan::gfx::wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
            let mut camera = Camera2D::new(&vec2(width, height));
            camera.update();

            let uniform_buffer =
                device.create_buffer_init(&tearchan::gfx::wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform Buffer"),
                    contents: bytemuck::cast_slice(camera.combine().as_slice()),
                    usage: tearchan::gfx::wgpu::BufferUsages::UNIFORM
                        | tearchan::gfx::wgpu::BufferUsages::COPY_DST,
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
                        resource: tearchan::gfx::wgpu::BindingResource::TextureView(
                            font_texture.view(),
                        ),
                    },
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 2,
                        resource: tearchan::gfx::wgpu::BindingResource::Sampler(
                            font_texture.sampler(),
                        ),
                    },
                ],
                label: None,
            });

            // Create the render pipeline
            let vertex_buffers = [
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

            let shader =
                device.create_shader_module(&tearchan::gfx::wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: get_standard_2d_shader_source(),
                });

            let pipeline =
                device.create_render_pipeline(&tearchan::gfx::wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: tearchan::gfx::wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &vertex_buffers,
                    },
                    fragment: Some(tearchan::gfx::wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[tearchan::gfx::wgpu::ColorTargetState {
                            format: context.gfx().surface_config.format,
                            blend: Some(tearchan::gfx::wgpu::BlendState {
                                color: tearchan::gfx::wgpu::BlendComponent {
                                    src_factor: tearchan::gfx::wgpu::BlendFactor::SrcAlpha,
                                    dst_factor: tearchan::gfx::wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: tearchan::gfx::wgpu::BlendOperation::Add,
                                },
                                alpha: tearchan::gfx::wgpu::BlendComponent {
                                    src_factor: tearchan::gfx::wgpu::BlendFactor::One,
                                    dst_factor: tearchan::gfx::wgpu::BlendFactor::One,
                                    operation: tearchan::gfx::wgpu::BlendOperation::Max,
                                },
                            }),
                            write_mask: tearchan::gfx::wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: tearchan::gfx::wgpu::PrimitiveState {
                        cull_mode: Some(tearchan::gfx::wgpu::Face::Back),
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: tearchan::gfx::wgpu::MultisampleState::default(),
                });

            Box::new(FontScene {
                font_texture,
                batch,
                bind_group,
                bind_group_layout,
                uniform_buffer,
                pipeline,
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

        let mut encoder = device
            .create_command_encoder(&tearchan::gfx::wgpu::CommandEncoderDescriptor { label: None });
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
            rpass.set_index_buffer(
                provider.index_buffer().slice(..),
                tearchan::gfx::wgpu::IndexFormat::Uint32,
            );
            rpass.set_vertex_buffer(0, provider.position_buffer().slice(..));
            rpass.set_vertex_buffer(1, provider.texcoord_buffer().slice(..));
            rpass.set_vertex_buffer(2, provider.color_buffer().slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw");
            rpass.draw_indexed(0..provider.index_count() as u32, 0, 0..1);
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
