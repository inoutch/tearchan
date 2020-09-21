use crate::scene::scene_context::SceneContext;
use crate::scene::scene_result::SceneResult;

pub mod scene_context;
pub mod scene_factory;
pub mod scene_manager;
pub mod scene_object;
pub mod scene_result;

pub trait Scene {
    fn update(&mut self, context: &mut SceneContext) -> SceneResult;

    // fn on_touch_start(&mut self, _touch: &Touch) {}
    //
    // fn on_touch_end(&mut self, _touch: &Touch) {}
    //
    // fn on_touch_move(&mut self, _touch: &Touch) {}
    //
    // fn on_touch_cancel(&mut self, _touch: &Touch) {}
    //
    // fn on_key_down(&mut self, _input: &KeyboardInput) {}
    //
    // fn on_key_up(&mut self, _input: &KeyboardInput) {}
    //
    // fn on_character(&mut self, _character: &char) {}
    //
    // fn on_resize(&mut self, _context: &mut ResizeContext) {}
}
