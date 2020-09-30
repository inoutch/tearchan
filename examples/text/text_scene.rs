use crate::text_object::TextObject;
use std::rc::Rc;
use tearchan::renderer::standard_font_renderer::StandardFontRenderer;
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
            include_bytes!("../../data/fonts/GenShinGothic-Medium.ttf").to_vec(),
            TextureConfig::default(),
            100.0f32,
        )
        .unwrap();
        let plugin = StandardFontRenderer::from_font_texture(&mut ctx.g.r, font_texture);
        ctx.plugin_manager_mut()
            .add(Box::new(plugin), "font".to_string(), 0);

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
