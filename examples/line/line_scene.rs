use crate::line::Line;
use std::rc::Rc;
use tearchan::plugin::object::camera::Camera2DDefaultObject;
use tearchan::plugin::renderer::standard_line_renderer::StandardLineRenderer;
use tearchan_core::game::object::GameObject;
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::camera::camera_2d::Camera2D;

const PRIMARY_CAMERA: &str = "primary";

pub struct LineScene {}

impl LineScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let mut plugin = Box::new(StandardLineRenderer::new(
                &mut ctx.g.r,
                PRIMARY_CAMERA.to_string(),
            ));

            plugin.register_caster_for_render_object(|object| {
                let casted = object.downcast_rc::<Line>().ok()?;
                Some(casted)
            });
            plugin.register_caster_for_camera(|object| {
                let casted = object.downcast_rc::<Camera2DDefaultObject>().ok()?;
                Some(casted)
            });
            ctx.plugin_manager_mut()
                .add(plugin, "renderer".to_string(), 0);

            let mut camera = Camera2D::new(&ctx.g.r.render_bundle().display_size().logical);
            camera.update();
            let camera_object = Camera2DDefaultObject::new(camera, PRIMARY_CAMERA.to_string());
            ctx.add(GameObject::new(Rc::new(camera_object)));
            ctx.add(GameObject::new(Rc::new(Line::default())));

            Box::new(LineScene {})
        }
    }
}

impl Scene for LineScene {
    fn update(&mut self, _context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
