use crate::skeleton_sprite::SkeletonSprite;
use std::rc::Rc;
use tearchan::plugin::animation::animation_runner::AnimationRunner;
use tearchan::plugin::renderer::sprite_renderer::SpriteRenderer;
use tearchan_core::game::object::GameObject;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::hal::backend::Texture;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;

pub struct SpriteScene {}

impl SpriteScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let image = Image::new_with_format(
                include_bytes!("../data/sprites/skeleton.png"),
                image::ImageFormat::Png,
            )
            .unwrap();
            let texture = Texture::new(ctx.g.r.render_bundle(), &image, TextureConfig::default());
            let plugin = SpriteRenderer::from_texture(&mut ctx.g.r, texture);
            ctx.plugin_manager_mut()
                .add(Box::new(plugin), "sprite".to_string(), 0);
            ctx.plugin_manager_mut().add(
                Box::new(AnimationRunner::new()),
                "animation".to_string(),
                0,
            );

            ctx.add(GameObject::new(Rc::new(SkeletonSprite::default())));

            Box::new(SpriteScene {})
        }
    }
}

impl Scene for SpriteScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
