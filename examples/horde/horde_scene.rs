use crate::horde_provider::HordeProvider;
use crate::person_object::PersonObject;
use crate::person_object_store::PersonObjectStore;
use nalgebra_glm::vec2;
use std::rc::Rc;
use tearchan::plugin::object::camera::Camera2DDefaultObject;
use tearchan::plugin::renderer::standard_2d_renderer::Standard2DRenderer;
use tearchan_core::game::object::GameObject;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::Texture;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;
use tearchan_horde::horde_plugin::HordePlugin;
use tearchan_horde::object::object_store::ObjectStore;

const PRIMARY_CAMERA: &str = "primary";

pub struct HordeScene {
    fps_counter: u64,
    fps_duration: f32,
}

impl HordeScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let image = Image::new(
                vec![
                    255u8, 0u8, 0u8, 255u8, 0u8, 255u8, 0u8, 255u8, 0u8, 0u8, 255u8, 255u8, 0u8,
                    0u8, 0u8, 0u8,
                ],
                vec2(2, 2),
            );
            let texture = Texture::new(ctx.g.r.render_bundle(), &image, TextureConfig::default());

            let renderer_plugin =
                Standard2DRenderer::from_texture(&mut ctx.g.r, texture, PRIMARY_CAMERA.to_string());
            ctx.plugin_manager_mut()
                .add(Box::new(renderer_plugin), "renderer".to_string(), 0);

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
                .add(Box::new(horde_plugin), "horde".to_string(), 1);

            let mut camera = Camera2D::new(&ctx.g.r.render_bundle().display_size().logical);
            camera.update();
            let camera_object = Camera2DDefaultObject::new(camera, PRIMARY_CAMERA.to_string());
            ctx.add(GameObject::new(Rc::new(camera_object)));

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
