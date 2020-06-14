use crate::core::scene::scene_context::SceneContext;
use crate::core::scene::touch::Touch;

pub trait SceneBase {
    fn update(&mut self, context: &mut SceneContext, delta: f32);
    fn on_touch_start(&self, touch: &Touch);
    fn on_touch_end(&self, touch: &Touch);
    fn on_touch_move(&self, touch: &Touch);
    fn on_touch_cancel(&self, touch: &Touch);
}
