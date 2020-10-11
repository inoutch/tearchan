use crate::text_object::TextObject;
use std::rc::Rc;
use tearchan::plugin::animation::animation_runner::AnimationRunner;
use tearchan::plugin::renderer::standard_font_renderer::StandardFontRenderer;
use tearchan_core::game::object::GameObject;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::hal::backend::FontTexture;
use tearchan_graphics::hal::texture::TextureConfig;

pub struct TextScene {}

impl TextScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| Box::new(TextScene::new(ctx))
    }

    pub fn new(ctx: &mut SceneContext) -> Self {
        let font_texture = FontTexture::new(
            ctx.g.r.render_bundle(),
            include_bytes!("../data/fonts/GenShinGothic-Medium.ttf").to_vec(),
            TextureConfig::default(),
            50.0f32,
        )
        .unwrap();

        let mut render_plugin = Box::new(StandardFontRenderer::from_font_texture(
            &mut ctx.g.r,
            font_texture,
        ));
        render_plugin.register_caster_for_render_object(|object| {
            let casted = object.downcast_rc::<TextObject>().ok()?;
            Some(casted)
        });
        ctx.plugin_manager_mut()
            .add(render_plugin, "font".to_string(), 0);

        let mut animation_plugin = AnimationRunner::new();
        animation_plugin.register(|object| {
            let casted = object.downcast_rc::<TextObject>().ok()?;
            Some(casted)
        });
        ctx.plugin_manager_mut()
            .add(Box::new(animation_plugin), "animator".to_string(), 0);

        ctx.add(GameObject::new(Rc::new(TextObject::new(
            "Example".to_string(),
        ))));
        TextScene {}
    }
}

impl Scene for TextScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
