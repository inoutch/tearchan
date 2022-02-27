use nalgebra_glm::{vec3, vec4, Mat4};
use rand::Rng;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::gfx::batch::types::{BatchTypeArray, BatchTypeTransform};
use tearchan::gfx::batch::v2::batch3d::BATCH3D_ATTRIBUTE_POSITION;
use tearchan::gfx::batch::v2::batch3d::{Batch3D, BATCH3D_ATTRIBUTE_NORMAL};
use tearchan::gfx::batch::v2::context::BatchContext;
use tearchan::gfx::batch::v2::object_manager::BatchObjectId;
use tearchan::gfx::camera::Camera3D;
use tearchan::gfx::texture::Texture;
use tearchan::gfx::wgpu::util::DeviceExt;
use tearchan::gfx::wgpu::TextureAspect;
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use tearchan::util::mesh::{Mesh, MeshBuilder};
use winit::event::{ElementState, TouchPhase, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;

#[allow(dead_code)]
struct Batch3DScene {
    batch: Batch3D,
    objects: VecDeque<BatchObjectId>,
    bind_group: tearchan::gfx::wgpu::BindGroup,
    transform_buffer: tearchan::gfx::wgpu::Buffer,
    light_position_buffer: tearchan::gfx::wgpu::Buffer,
    pipeline: tearchan::gfx::wgpu::RenderPipeline,
    depth_texture: Texture,
    camera: Camera3D,
    camera_rotation: f32,
    light_rotation: f32,
    light_object: BatchObjectId,
}

impl Batch3DScene {
    pub fn factory() -> SceneFactory {
        |context, _| {
            let gfx = context.gfx();
            let device = gfx.device;
            let queue = gfx.queue;
            let width = context.gfx().surface_config.width as f32;
            let height = context.gfx().surface_config.height as f32;
            let aspect = width / height;

            let objects = VecDeque::new();
            let mut batch = Batch3D::new(device);

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
                        tearchan::gfx::wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: tearchan::gfx::wgpu::ShaderStages::FRAGMENT,
                            ty: tearchan::gfx::wgpu::BindingType::Buffer {
                                ty: tearchan::gfx::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        tearchan::gfx::wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: tearchan::gfx::wgpu::ShaderStages::FRAGMENT,
                            ty: tearchan::gfx::wgpu::BindingType::Buffer {
                                ty: tearchan::gfx::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        tearchan::gfx::wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: tearchan::gfx::wgpu::ShaderStages::FRAGMENT,
                            ty: tearchan::gfx::wgpu::BindingType::Buffer {
                                ty: tearchan::gfx::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
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
            let texture_view =
                texture.create_view(&tearchan::gfx::wgpu::TextureViewDescriptor::default());
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
            camera.position = vec3(0.0f32, 1.0f32, 4.0f32);
            camera.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            camera.up = vec3(0.0f32, 1.0f32, 0.0f32);
            camera.update();

            let transform_buffer =
                device.create_buffer_init(&tearchan::gfx::wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform Buffer"),
                    contents: bytemuck::cast_slice(camera.combine().as_slice()),
                    usage: tearchan::gfx::wgpu::BufferUsages::UNIFORM
                        | tearchan::gfx::wgpu::BufferUsages::COPY_DST,
                });
            let light_ambient_strength_buffer =
                device.create_buffer_init(&tearchan::gfx::wgpu::util::BufferInitDescriptor {
                    label: Some("LightAmbientBuffer"),
                    contents: bytemuck::bytes_of(&0.13f32),
                    usage: tearchan::gfx::wgpu::BufferUsages::UNIFORM
                        | tearchan::gfx::wgpu::BufferUsages::COPY_DST,
                });
            let light_color_buffer =
                device.create_buffer_init(&tearchan::gfx::wgpu::util::BufferInitDescriptor {
                    label: Some("LightColorBuffer"),
                    contents: bytemuck::cast_slice(vec4(1.0f32, 1.0f32, 1.0f32, 0.0f32).as_slice()),
                    usage: tearchan::gfx::wgpu::BufferUsages::UNIFORM
                        | tearchan::gfx::wgpu::BufferUsages::COPY_DST,
                });
            let light_position_buffer =
                device.create_buffer_init(&tearchan::gfx::wgpu::util::BufferInitDescriptor {
                    label: Some("LightPositionBuffer"),
                    contents: bytemuck::cast_slice(
                        vec4(0.0f32, 10.0f32, 0.0f32, 0.0f32).as_slice(),
                    ),
                    usage: tearchan::gfx::wgpu::BufferUsages::UNIFORM
                        | tearchan::gfx::wgpu::BufferUsages::COPY_DST,
                });
            let depth_texture = Texture::new_depth_texture(
                device,
                gfx.surface_config.width,
                gfx.surface_config.height,
                "DepthTexture",
            );

            // Create bind group
            let bind_group = device.create_bind_group(&tearchan::gfx::wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 0,
                        resource: transform_buffer.as_entire_binding(),
                    },
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 1,
                        resource: tearchan::gfx::wgpu::BindingResource::TextureView(&texture_view),
                    },
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 2,
                        resource: tearchan::gfx::wgpu::BindingResource::Sampler(&sampler),
                    },
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 3,
                        resource: light_ambient_strength_buffer.as_entire_binding(),
                    },
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 4,
                        resource: light_color_buffer.as_entire_binding(),
                    },
                    tearchan::gfx::wgpu::BindGroupEntry {
                        binding: 5,
                        resource: light_position_buffer.as_entire_binding(),
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
                tearchan::gfx::wgpu::VertexBufferLayout {
                    array_stride: 3 * std::mem::size_of::<f32>() as u64, // normals
                    step_mode: tearchan::gfx::wgpu::VertexStepMode::Vertex,
                    attributes: &[tearchan::gfx::wgpu::VertexAttribute {
                        format: tearchan::gfx::wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 3,
                    }],
                },
            ];

            let shader =
                device.create_shader_module(&tearchan::gfx::wgpu::include_wgsl!("./batch3d.wgsl"));

            let pipeline =
                device.create_render_pipeline(&tearchan::gfx::wgpu::RenderPipelineDescriptor {
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
                    depth_stencil: Some(tearchan::gfx::wgpu::DepthStencilState {
                        format: depth_texture.format(),
                        depth_write_enabled: true,
                        depth_compare: tearchan::gfx::wgpu::CompareFunction::Less,
                        stencil: tearchan::gfx::wgpu::StencilState::default(),
                        bias: tearchan::gfx::wgpu::DepthBiasState::default(),
                    }),
                    multisample: tearchan::gfx::wgpu::MultisampleState::default(),
                });

            let mesh = load_obj_mesh("cube.obj");
            let light_object = batch.add(
                BatchTypeArray::V1U32 { data: mesh.indices },
                vec![
                    BatchTypeArray::V3F32 {
                        data: mesh.positions,
                    },
                    BatchTypeArray::V2F32 {
                        data: mesh.texcoords,
                    },
                    BatchTypeArray::V4F32 { data: mesh.colors },
                    BatchTypeArray::V3F32 { data: mesh.normals },
                ],
                None,
            );

            Box::new(Batch3DScene {
                batch,
                objects,
                bind_group,
                transform_buffer,
                light_position_buffer,
                pipeline,
                depth_texture,
                camera,
                camera_rotation: 0.0f32,
                light_rotation: 0.0f32,
                light_object,
            })
        }
    }
}

impl Scene for Batch3DScene {
    fn update(&mut self, context: &mut SceneContext, event: WindowEvent) -> SceneControlFlow {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    if input.state == ElementState::Pressed {
                        match key {
                            VirtualKeyCode::Right => {
                                create_object(&mut self.batch, &mut self.objects);
                            }
                            VirtualKeyCode::Left => {
                                destroy_object(&mut self.batch, &mut self.objects);
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
                        create_object(&mut self.batch, &mut self.objects);
                    } else {
                        destroy_object(&mut self.batch, &mut self.objects);
                    }
                }
            }
            _ => {}
        }
        SceneControlFlow::None
    }

    fn render(&mut self, context: &mut SceneRenderContext) -> SceneControlFlow {
        self.camera_rotation += context.delta;
        self.camera.position = vec3(
            self.camera_rotation.cos() * 4.0f32,
            1.0f32,
            self.camera_rotation.sin() * 4.0f32,
        );
        self.camera.update();
        self.light_rotation += context.delta;
        let light_position = vec3(0.0f32, self.light_rotation.cos() * 2.0f32, 0.0f32);
        self.batch.transform(
            self.light_object,
            BATCH3D_ATTRIBUTE_POSITION,
            BatchTypeTransform::Mat4F32 {
                m: nalgebra_glm::scale(
                    &nalgebra_glm::translation(&light_position),
                    &vec3(0.2f32, 0.2f32, 0.2f32),
                ),
            },
        );

        let queue = context.gfx().queue;
        let device = context.gfx().device;

        queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(self.camera.combine().as_slice()),
        );
        queue.write_buffer(
            &self.light_position_buffer,
            0,
            bytemuck::cast_slice(light_position.as_slice()),
        );

        let mut encoder = device
            .create_command_encoder(&tearchan::gfx::wgpu::CommandEncoderDescriptor { label: None });
        {
            self.batch.flush(BatchContext {
                device,
                queue,
                encoder: &mut encoder,
            });
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
                depth_stencil_attachment: Some(
                    tearchan::gfx::wgpu::RenderPassDepthStencilAttachment {
                        view: self.depth_texture.view(),
                        depth_ops: Some(tearchan::gfx::wgpu::Operations {
                            load: tearchan::gfx::wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    },
                ),
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            self.batch.bind(&mut rpass);
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..index_count, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        SceneControlFlow::None
    }
}

fn load_obj_mesh<P: AsRef<Path>>(filepath: P) -> Mesh {
    let mut path = PathBuf::new();
    path.push(std::env::current_dir().unwrap());
    path.push("tearchan-examples/batch3d");
    path.push(filepath);

    let obj_str = read_to_string(path).unwrap();
    let object_set = wavefront_obj::obj::parse(obj_str).unwrap();
    MeshBuilder::new()
        .with_object(object_set.objects.first().unwrap())
        .build()
        .unwrap()
}

fn create_object(batch: &mut Batch3D, objects: &mut VecDeque<BatchObjectId>) {
    let mut rng = rand::thread_rng();
    let models = vec!["cube.obj", "icosphere.obj", "suzanne.obj"];
    let mesh = load_obj_mesh(models.get(rng.gen_range(0..models.len())).unwrap());
    let id = batch.add(
        BatchTypeArray::V1U32 { data: mesh.indices },
        vec![
            BatchTypeArray::V3F32 {
                data: mesh.positions,
            },
            BatchTypeArray::V2F32 {
                data: mesh.texcoords,
            },
            BatchTypeArray::V4F32 { data: mesh.colors },
            BatchTypeArray::V3F32 { data: mesh.normals },
        ],
        None,
    );
    let rotation_x = rng.gen_range(0.0f32..PI * 2.0f32);
    let rotation_y = rng.gen_range(0.0f32..PI * 2.0f32);
    let rotation_z = rng.gen_range(0.0f32..PI * 2.0f32);
    let position_transform = BatchTypeTransform::Mat4F32 {
        m: nalgebra_glm::scale(
            &nalgebra_glm::rotate_z(
                &nalgebra_glm::rotate_y(
                    &nalgebra_glm::rotate_x(
                        &nalgebra_glm::translation(&vec3(
                            rng.gen_range(-1.0f32..1.0f32),
                            rng.gen_range(-1.0f32..1.0f32),
                            rng.gen_range(-1.0f32..1.0f32),
                        )),
                        rotation_x,
                    ),
                    rotation_y,
                ),
                rotation_z,
            ),
            &vec3(0.2f32, 0.2f32, 0.2f32),
        ),
    };
    let normal_transform = BatchTypeTransform::Mat4F32 {
        m: nalgebra_glm::rotate_z(
            &nalgebra_glm::rotate_y(
                &nalgebra_glm::rotate_x(&Mat4::identity(), rotation_x),
                rotation_y,
            ),
            rotation_z,
        ),
    };
    batch.transform(id, BATCH3D_ATTRIBUTE_POSITION, position_transform);
    batch.transform(id, BATCH3D_ATTRIBUTE_NORMAL, normal_transform);
    objects.push_back(id);
}

fn destroy_object(batch: &mut Batch3D, sprites: &mut VecDeque<BatchObjectId>) {
    let id = match sprites.pop_front() {
        None => return,
        Some(id) => id,
    };
    batch.remove(id);
}

fn main() {
    env_logger::init();

    let window_builder = WindowBuilder::new().with_title("empty");
    let startup_config = EngineStartupConfigBuilder::new()
        .window_builder(window_builder)
        .scene_factory(Batch3DScene::factory())
        .build();
    let engine = Engine::new(startup_config);
    engine.run();
}
