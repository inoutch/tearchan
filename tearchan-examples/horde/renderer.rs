use crate::create_texture_view;
use crate::utils::SCALE_SIZE;
use nalgebra_glm::{translate, vec3, vec4, Mat4, Vec2, Vec3};
use std::collections::HashMap;
use tearchan::gfx::batch::batch_billboard::{
    BatchBillboard, BATCH_BILLBOARD_ATTRIBUTE_COLOR, BATCH_BILLBOARD_ATTRIBUTE_ORIGIN,
};
use tearchan::gfx::batch::batch_line::BatchLine;
use tearchan::gfx::batch::context::BatchContext;
use tearchan::gfx::batch::object_manager::BatchObjectId;
use tearchan::gfx::batch::types::{BatchTypeArray, BatchTypeTransform};
use tearchan::gfx::camera::{Billboard, Camera3D};
use tearchan::gfx::material::material_billboard::{MaterialBillboard, MaterialBillboardParams};
use tearchan::gfx::material::material_line::{MaterialLine, MaterialLineParams};
use tearchan::gfx::texture::Texture;
use tearchan::gfx::uniform_buffer::UniformBuffer;
use tearchan::gfx::wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, SamplerDescriptor,
};
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::util::math::rect::rect2;
use tearchan::util::math::vec::vec4_white;
use tearchan::util::mesh::MeshBuilder;
use tearchan_ecs::component::EntityId;

pub struct Renderer {
    #[allow(dead_code)]
    lines: HashMap<EntityId, Vec<BatchObjectId>>,
    sprites: HashMap<EntityId, BatchObjectId>,
    // renderings
    camera: Camera3D,
    batch_billboard: BatchBillboard,
    material_billboard: MaterialBillboard,
    batch_line: BatchLine,
    material_line: MaterialLine,
    depth_texture: Texture,
    transform_buffer: UniformBuffer<Mat4>,
    billboard_buffer: UniformBuffer<Billboard>,
}

impl Renderer {
    pub fn new(context: &SceneContext) -> Renderer {
        let aspect =
            context.gfx().surface_config.width as f32 / context.gfx().surface_config.height as f32;
        let device = context.gfx().device;
        let queue = context.gfx().queue;

        let batch_billboard = BatchBillboard::new(device);
        let batch_line = BatchLine::new(device);

        let mut camera = Camera3D::new(aspect, 0.1f32, 10.0f32);
        camera.position = vec3(25.0f32 * SCALE_SIZE, 2.0f32, 50.0f32 * SCALE_SIZE);
        camera.target_position = vec3(25.0f32 * SCALE_SIZE, 0.0f32, 25.0f32 * SCALE_SIZE);
        camera.update();

        let depth_texture = Texture::new_depth_texture(
            device,
            context.gfx().surface_config.width,
            context.gfx().surface_config.height,
            "DepthTexture",
        );

        let transform_buffer = UniformBuffer::new(device, camera.combine());
        let billboard_buffer = UniformBuffer::new(device, &camera.base().billboard());

        let texture_view = create_texture_view(device, queue);
        let sampler = device.create_sampler(&SamplerDescriptor::default());

        let material_billboard = MaterialBillboard::new(
            context.gfx().device,
            MaterialBillboardParams {
                transform_buffer: transform_buffer.buffer(),
                camera_buffer: billboard_buffer.buffer(),
                texture_view: &texture_view,
                sampler: &sampler,
                color_format: context.gfx().surface_config.format,
                depth_format: depth_texture.format(),
                shader_module: None,
            },
        );
        let material_line = MaterialLine::new(
            device,
            MaterialLineParams {
                transform_buffer: transform_buffer.buffer(),
                color_format: context.gfx().surface_config.format,
                depth_format: None,
                shader_module: None,
            },
        );

        Renderer {
            lines: Default::default(),
            sprites: Default::default(),
            camera,
            batch_billboard,
            material_billboard,
            batch_line,
            material_line,
            depth_texture,
            transform_buffer,
            billboard_buffer,
        }
    }

    pub fn render(&mut self, context: &mut SceneRenderContext) {
        let queue = context.gfx().queue;
        let device = context.gfx().device;

        self.camera.update();

        self.transform_buffer.write(queue, self.camera.combine());
        self.billboard_buffer
            .write(queue, &self.camera.base().billboard());

        self.batch_billboard.flush(BatchContext { device, queue });
        self.batch_line.flush(BatchContext { device, queue });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let index_count = self.batch_billboard.index_count() as u32;
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
            self.material_billboard.bind(&mut rpass);
            self.batch_billboard.bind(&mut rpass);
            rpass.draw_indexed(0..index_count, 0, 0..1);
        }
        {
            let index_count = self.batch_line.index_count() as u32;
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &context.gfx_rendering().view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.material_line.bind(&mut rpass);
            self.batch_line.bind(&mut rpass);
            rpass.draw_indexed(0..index_count, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }

    pub fn resize(&mut self, context: &SceneContext) {
        let width = context.gfx().surface_config.width.max(1);
        let height = context.gfx().surface_config.height.max(1);
        let aspect = width as f32 / height as f32;
        let mut camera = Camera3D::default_with_aspect(aspect);
        camera.position = self.camera.position;
        camera.target_position = self.camera.target_position;
        camera.update();
        self.camera = camera;
        self.depth_texture =
            Texture::new_depth_texture(context.gfx().device, width, height, "DepthTexture");
    }

    pub fn add_sprite(&mut self, entity_id: EntityId, position: &Vec2) {
        let mesh = MeshBuilder::new()
            .with_rect(&rect2(-0.01f32, -0.01f32, 0.02f32, 0.02f32))
            .build()
            .unwrap();
        let origins = mesh.positions.to_vec();
        let id = self.batch_billboard.add(
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
        self.batch_billboard.transform(
            id,
            BATCH_BILLBOARD_ATTRIBUTE_ORIGIN,
            BatchTypeTransform::Mat4F32 {
                m: translate(&Mat4::identity(), &vec3(position.x, 0.0f32, position.y)),
            },
        );
        self.sprites.insert(entity_id, id);
    }

    pub fn add_line(&mut self, entity_id: EntityId, lines: &Vec<(Vec2, Vec2)>) {
        let mut ids = Vec::new();
        for (from, to) in lines {
            let id = self.batch_line.add(
                BatchTypeArray::V1U32 { data: vec![0, 1] },
                vec![
                    BatchTypeArray::V3F32 {
                        data: vec![vec3(from.x, 0.0f32, from.y), vec3(to.x, 0.0f32, to.y)],
                    },
                    BatchTypeArray::V4F32 {
                        data: vec![vec4_white(), vec4_white()],
                    },
                ],
                None,
            );
            ids.push(id);
        }
        self.lines.insert(entity_id, ids);
    }

    pub fn update_sprite_position(&mut self, entity_id: EntityId, position: &Vec2) {
        if let Some(id) = self.sprites.get(&entity_id) {
            self.batch_billboard.transform(
                *id,
                BATCH_BILLBOARD_ATTRIBUTE_ORIGIN,
                BatchTypeTransform::Mat4F32 {
                    m: translate(&Mat4::identity(), &vec3(position.x, 0.0f32, position.y)),
                },
            );
        }
    }

    pub fn update_sprite_color(&mut self, entity_id: EntityId, color: &Vec3) {
        if let Some(id) = self.sprites.get(&entity_id) {
            self.batch_billboard.rewrite_vertices(
                *id,
                BATCH_BILLBOARD_ATTRIBUTE_COLOR,
                BatchTypeArray::V4F32 {
                    data: vec![vec4(color.x, color.y, color.z, 1.0f32); 4],
                },
            );
        }
    }
}
