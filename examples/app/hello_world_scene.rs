use tearchan::core::graphic::hal::backend::FixedVertexBuffer;
use tearchan::core::graphic::shader::shader_source::ShaderSource;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;

pub struct HelloWorldScene {}

impl HelloWorldScene {
    pub fn creator() -> Option<SceneCreator> {
        Some(|scene_context| {
            let api = scene_context.borrow_renderer_api();

            // let screen_size = api.screen_size();
            // let camera = Camera::new(screen_size);

            // let texture = Rc::new(Texture::empty());
            // let material = Rc::new(Material::new(texture));
            // let graphic_pipeline = GraphicPipeline::new(material);

            // let batch = Batch2D::new();
            // let polygon = Polygon2D::new(vec2(0.0f, 0.0f), vec2(32.0f, 32.0f));
            // batch.add(polygon);

            Box::new(HelloWorldScene {})
        })
    }
}

impl SceneBase for HelloWorldScene {
    fn update(&mut self, _scene_context: &mut SceneContext, _delta: f32) {
        // self.batch.render(&self.graphic_pipeline);
    }
}
