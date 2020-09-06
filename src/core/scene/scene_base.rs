use crate::core::graphic::hal::renderer::ResizeContext;
use crate::core::scene::scene_context::SceneContext;
use crate::core::scene::touch::Touch;
use winit::event::KeyboardInput;

pub trait SceneBase {
    fn update(&mut self, context: &mut SceneContext, delta: f32);
    fn on_touch_start(&mut self, _touch: &Touch) {}
    fn on_touch_end(&mut self, _touch: &Touch) {}
    fn on_touch_move(&mut self, _touch: &Touch) {}
    fn on_touch_cancel(&mut self, _touch: &Touch) {}
    fn on_key_down(&mut self, _input: &KeyboardInput) {}
    fn on_key_up(&mut self, _input: &KeyboardInput) {}
    fn on_character(&mut self, _character: &char) {}
    fn on_resize(&mut self, _context: &mut ResizeContext) {}
}
