use crate::cube::Cube;
use nalgebra_glm::vec3;
use std::rc::Rc;
use tearchan::plugin::animation::animation_runner::AnimationRunner;
use tearchan::plugin::renderer::standard_3d_renderer::Standard3DRenderer;
use tearchan_core::game::object::GameObject;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::hal::backend::Texture;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;

pub struct CubeScene {}

impl CubeScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let image = Image::new_empty();
            let texture = Texture::new(ctx.g.r.render_bundle(), &image, TextureConfig::default());
            let mut plugin = Standard3DRenderer::from_texture(&mut ctx.g.r, texture);
            plugin.provider_mut().camera_mut().position = vec3(0.0f32, 2.0f32, 4.0f32);
            plugin.provider_mut().camera_mut().target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            plugin.provider_mut().camera_mut().up = vec3(0.0f32, 1.0f32, 0.0f32);
            plugin.provider_mut().camera_mut().update();

            ctx.plugin_manager_mut()
                .add(Box::new(plugin), "renderer".to_string(), 0);
            ctx.plugin_manager_mut().add(
                Box::new(AnimationRunner::new()),
                "animation".to_string(),
                0,
            );

            ctx.add(GameObject::new(Rc::new(Cube::default())));

            Box::new(CubeScene {})
        }
    }
}

impl Scene for CubeScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
