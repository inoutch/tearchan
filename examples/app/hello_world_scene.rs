use nalgebra_glm::vec2;
use tearchan::core::graphic::batch::batch2d::Batch2D;
use tearchan::core::graphic::batch::batch_buffer_f32::BatchBufferF32;
use tearchan::core::graphic::batch::default::Batch;
use tearchan::core::graphic::hal::image::Image;
use tearchan::core::graphic::polygon::default::Polygon;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::extension::shared::Shared;
use tearchan::math::mesh::MeshBuilder;

pub struct HelloWorldScene {
    batch: Batch<Polygon, BatchBufferF32, Batch2D<BatchBufferF32>>,
}

impl HelloWorldScene {
    pub fn creator() -> Option<SceneCreator> {
        Some(|scene_context| {
            let image = Image::new_empty();
            let texture = scene_context.renderer_api.create_texture(&image);
            let screen_size = scene_context.renderer_api.screen_size();
            // let camera = Camera2D::new(screen_size);

            // let texture = Rc::new(Texture::empty());
            // let material = Rc::new(Material::new(texture));
            // let graphic_pipeline = GraphicPipeline::new(material);

            let mesh = MeshBuilder::new()
                .with_square(vec2(32.0f32, 32.0f32))
                .build()
                .unwrap();

            let mut batch = Batch2D::new(scene_context.renderer_api);
            let polygon = Shared::new(Polygon::new(mesh));
            batch.add(&polygon, 0);

            Box::new(HelloWorldScene { batch })
        })
    }
}

impl SceneBase for HelloWorldScene {
    fn update(&mut self, _scene_context: &mut SceneContext, _delta: f32) {
        // self.batch.render(&self.graphic_pipeline);
        self.batch.render();
    }
}
