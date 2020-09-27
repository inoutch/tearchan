use nalgebra_glm::{vec2, vec3};
use std::thread;
use tearchan::renderer::standard_2d_renderer::Standard2DRenderer;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::batch::batch_command::{BatchCommand, BatchCommandData, BATCH_ID_EMPTY};
use tearchan_graphics::hal::backend::Texture;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;
use tearchan_utility::mesh::MeshBuilder;

pub struct CubeScene {}

impl CubeScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let image = Image::new_empty();
            let texture = Texture::new(ctx.g.r.render_bundle(), &image, TextureConfig::default());

            let mut plugin = Box::new(Standard2DRenderer::new(&mut ctx.g.r, texture));

            let mut batch_queue = plugin.create_batch_queue();
            thread::spawn(move || {
                let mesh = MeshBuilder::new()
                    .with_square(vec2(100.0f32, 100.0f32))
                    .build()
                    .unwrap();
                let batch_object_id = batch_queue
                    .queue(BatchCommand::Add {
                        id: BATCH_ID_EMPTY,
                        data: vec![
                            BatchCommandData::V3U32 {
                                data: vec![vec3(0u32, 3u32, 2u32), vec3(0u32, 1u32, 3u32)],
                            },
                            BatchCommandData::V3F32 {
                                data: mesh.positions.clone(),
                            },
                            BatchCommandData::V4F32 {
                                data: mesh.colors.clone(),
                            },
                            BatchCommandData::V2F32 {
                                data: mesh.texcoords,
                            },
                        ],
                        order: None,
                    })
                    .unwrap();
                batch_queue.queue(BatchCommand::Remove {
                    id: batch_object_id,
                });
            });

            ctx.plugin_manager_mut()
                .add(plugin, "renderer".to_string(), 0);

            Box::new(CubeScene {})
        }
    }
}

impl Scene for CubeScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
