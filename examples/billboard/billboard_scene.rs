use crate::skeleton_billboard::SkeletonBillboard;
use nalgebra_glm::vec3;
use std::rc::Rc;
use tearchan::plugin::renderer::billboard_renderer::BillboardRenderer;
use tearchan_core::game::object::GameObject;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::hal::backend::Texture;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;

pub struct BillboardScene {}

impl BillboardScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let image = Image::new_with_format(
                include_bytes!("../data/sprites/skeleton.png"),
                image::ImageFormat::Png,
            )
            .unwrap();
            let texture = Texture::new(ctx.g.r.render_bundle(), &image, TextureConfig::default());
            let mut plugin = Box::new(BillboardRenderer::from_texture(&mut ctx.g.r, texture));
            plugin.register_caster_for_billboard(|object| {
                let casted = object.downcast_rc::<SkeletonBillboard>().ok()?;
                Some(casted)
            });
            plugin.provider_mut().camera_mut().position = vec3(0.0f32, 2.0f32, 4.0f32);
            plugin.provider_mut().camera_mut().target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            plugin.provider_mut().camera_mut().up = vec3(0.0f32, 1.0f32, 0.0f32);
            plugin.provider_mut().camera_mut().update();

            ctx.plugin_manager_mut()
                .add(plugin, "billboard".to_string(), 0);

            let skeleton = SkeletonBillboard::default();
            ctx.add(GameObject::new(Rc::new(skeleton)));

            Box::new(BillboardScene {})
        }
    }
}

impl Scene for BillboardScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
