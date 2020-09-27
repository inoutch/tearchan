use crate::horde_provider::HordeProvider;
use crate::person_object::PersonObject;
use crate::person_object_store::PersonObjectStore;
use std::rc::Rc;
use tearchan::renderer::standard_2d_renderer::Standard2DRenderer;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::hal::backend::Texture;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;
use tearchan_horde::horde_plugin::HordePlugin;
use tearchan_horde::object::object_store::ObjectStore;

pub struct HordeScene {
    fps_counter: u64,
    fps_duration: f32,
}

impl HordeScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let image = Image::new_empty();
            let texture = Texture::new(ctx.g.r.render_bundle(), &image, TextureConfig::default());

            let renderer_plugin = Standard2DRenderer::new(&mut ctx.g.r, texture);

            let mut horde_plugin = HordePlugin::new(
                HordeProvider::default(),
                ctx.plugin_manager_mut().create_operator(),
            );
            horde_plugin.register_factory(PersonObject::kind(), PersonObject::factory());
            horde_plugin
                .create_object(ObjectStore::new(
                    PersonObject::kind().to_string(),
                    Rc::new(PersonObjectStore::default()),
                ))
                .unwrap();

            ctx.plugin_manager_mut()
                .add(Box::new(horde_plugin), "horde".to_string(), 0);
            ctx.plugin_manager_mut()
                .add(Box::new(renderer_plugin), "renderer".to_string(), 0);

            Box::new(HordeScene {
                fps_counter: 0,
                fps_duration: 0.0f32,
            })
        }
    }
}

impl Scene for HordeScene {
    fn update(&mut self, context: &mut SceneContext) -> SceneResult {
        self.fps_counter += 1;
        self.fps_duration += context.g.delta;
        if self.fps_counter > 300 {
            let avg = self.fps_duration / self.fps_counter as f32;
            println!("FPS: {}", 1.0f32 / avg);
            self.fps_counter = 0;
            self.fps_duration = 0.0f32;
        }

        SceneResult::None
    }
}