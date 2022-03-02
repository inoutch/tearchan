use nalgebra_glm::vec3;
use rand::Rng;
use std::collections::VecDeque;
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::gfx::batch::types::BatchTypeArray;
use tearchan::gfx::batch::v2::batch_billboard::BatchBillboard;
use tearchan::gfx::batch::v2::context::BatchContext;
use tearchan::gfx::batch::v2::object_manager::BatchObjectId;
use tearchan::gfx::camera::Camera3D;
use tearchan::gfx::material::material_billboard::{MaterialBillboard, MaterialBillboardParams};
use tearchan::gfx::texture::Texture;
use tearchan::gfx::wgpu::util::{BufferInitDescriptor, DeviceExt};
use tearchan::gfx::wgpu::{
    Buffer, BufferUsages, Color, CommandEncoderDescriptor, Device, Extent3d, ImageCopyTexture,
    ImageDataLayout, LoadOp, Operations, Origin3d, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, SamplerDescriptor, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use tearchan::util::math::rect::rect2;
use tearchan::util::mesh::MeshBuilder;
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;

struct BatchBillboardScene {
    camera: Camera3D,
    camera_rotation: f32,
    batch: BatchBillboard,
    material: MaterialBillboard,
    depth_texture: Texture,
    transform_buffer: Buffer,
    billboard_buffer: Buffer,
    squares: VecDeque<BatchObjectId>,
}

impl BatchBillboardScene {
    pub fn factory() -> SceneFactory {
        |context, _| {
            let aspect = context.gfx().surface_config.width as f32
                / context.gfx().surface_config.height as f32;
            let device = context.gfx().device;
            let queue = context.gfx().queue;
            let batch = BatchBillboard::new(device);
            let mut camera = Camera3D::new(aspect, 0.1f32, 10.0f32);
            camera.position = vec3(0.0f32, 1.0f32, 1.0f32);
            camera.update();

            let depth_texture = Texture::new_depth_texture(
                device,
                context.gfx().surface_config.width,
                context.gfx().surface_config.height,
                "DepthTexture",
            );

            let transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("TransformBuffer"),
                contents: bytemuck::cast_slice(camera.combine().as_slice()),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
            let billboard_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("BillboardBuffer"),
                contents: bytemuck::bytes_of(&camera.base().billboard()),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let texture_view = create_texture_view(device, queue);
            let sampler = device.create_sampler(&SamplerDescriptor::default());

            let material = MaterialBillboard::new(
                context.gfx().device,
                MaterialBillboardParams {
                    transform_buffer: &transform_buffer,
                    camera_buffer: &billboard_buffer,
                    texture_view: &texture_view,
                    sampler: &sampler,
                    color_format: context.gfx().surface_config.format,
                    depth_format: depth_texture.format(),
                    shader_module: None,
                },
            );
            Box::new(BatchBillboardScene {
                camera,
                camera_rotation: 0.0f32,
                batch,
                material,
                depth_texture,
                transform_buffer,
                billboard_buffer,
                squares: VecDeque::new(),
            })
        }
    }

    pub fn add_square(&mut self) {
        let mut rng = rand::thread_rng();
        let position = vec3(
            rng.gen_range(-1.0f32..1.0f32),
            0.0f32,
            rng.gen_range(-1.0f32..1.0f32),
        );
        let mesh = MeshBuilder::new()
            .with_rect(&rect2(-0.05f32, -0.05f32, 0.1f32, 0.1f32))
            .build()
            .unwrap();
        let origins = mesh
            .positions
            .iter()
            .map(|_| position.clone_owned())
            .collect();
        let id = self.batch.add(
            BatchTypeArray::V1U32 { data: mesh.indices },
            vec![
                BatchTypeArray::V3F32 {
                    data: mesh.positions,
                },
                BatchTypeArray::V2F32 {
                    data: mesh.texcoords,
                },
                BatchTypeArray::V4F32 { data: mesh.colors },
                BatchTypeArray::V3F32 { data: origins },
            ],
            None,
        );
        self.squares.push_back(id);
    }

    pub fn remove_square(&mut self) {
        if let Some(id) = self.squares.pop_front() {
            self.batch.remove(id);
        }
    }
}

impl Scene for BatchBillboardScene {
    fn update(&mut self, context: &mut SceneContext, event: WindowEvent) -> SceneControlFlow {
        match event {
            WindowEvent::Resized(size) => {
                let width = size.width.max(1);
                let height = size.height.max(1);
                let aspect = width as f32 / height as f32;
                self.camera = Camera3D::default_with_aspect(aspect);
                self.camera.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
                self.camera.up = vec3(0.0f32, 1.0f32, 0.0f32);
                self.depth_texture =
                    Texture::new_depth_texture(context.gfx().device, width, height, "DepthTexture");
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    if input.state == ElementState::Pressed {
                        match key {
                            VirtualKeyCode::Right => self.add_square(),
                            VirtualKeyCode::Left => self.remove_square(),
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

        self.camera_rotation += context.delta * 0.5f32;
        self.camera.position = vec3(
            1.0f32 * self.camera_rotation.cos(),
            1.0f32,
            1.0f32 * self.camera_rotation.sin(),
        );
        self.camera.update();

        queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(self.camera.combine().as_slice()),
        );
        queue.write_buffer(
            &self.billboard_buffer,
            0,
            bytemuck::bytes_of(&self.camera.base().billboard()),
        );

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
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
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

    let window_builder = WindowBuilder::new().with_title("Billboard example");
    let startup_config = EngineStartupConfigBuilder::new()
        .window_builder(window_builder)
        .scene_factory(BatchBillboardScene::factory())
        .build();
    let engine = Engine::new(startup_config);
    engine.run();
}

fn create_texture_view(device: &Device, queue: &Queue) -> TextureView {
    let size = 2u32;
    let texel = vec![
        0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
    ];
    let texture_extent = Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING
            | TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::COPY_DST,
    });
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    queue.write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &texel,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(std::num::NonZeroU32::new(4 * size).unwrap()),
            rows_per_image: None,
        },
        texture_extent,
    );
    texture_view
}
