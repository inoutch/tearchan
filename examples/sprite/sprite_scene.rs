use crate::skeleton_sprite::SkeletonSprite;
use std::rc::Rc;
use tearchan::plugin::animation::animation_runner::AnimationRunner;
use tearchan::plugin::object::camera::Camera2DDefaultObject;
use tearchan::plugin::renderer::sprite_renderer::SpriteRenderer;
use tearchan_core::game::object::GameObject;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::hal::backend::Texture;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;

const PRIMARY_CAMERA: &str = "primary";

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
            let mut render_plugin =
                SpriteRenderer::from_texture(&mut ctx.g.r, texture, PRIMARY_CAMERA.to_string());
            render_plugin.register_caster_for_render_object(|object| {
                let casted = object.downcast_rc::<SkeletonSprite>().ok()?;
                Some(casted)
            });
            render_plugin.register_caster_for_camera(|object| {
                let casted = object.downcast_rc::<Camera2DDefaultObject>().ok()?;
                Some(casted)
            });

            ctx.plugin_manager_mut()
                .add(Box::new(render_plugin), "sprite".to_string(), 0);

            let mut plugin = AnimationRunner::new();
            plugin.register(|object| {
                let casted = object.downcast_rc::<SkeletonSprite>().ok()?;
                Some(casted)
            });
            ctx.plugin_manager_mut()
                .add(Box::new(plugin), "animation".to_string(), 0);

            let mut camera = Camera2D::new(&ctx.g.r.render_bundle().display_size().logical);
            camera.update();
            let camera_object = Camera2DDefaultObject::new(camera, PRIMARY_CAMERA.to_string());
            ctx.add(GameObject::new(Rc::new(camera_object)));
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
