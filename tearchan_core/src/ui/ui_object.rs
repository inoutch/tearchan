use crate::game::object::game_object_base::GameObjectBase;
use crate::ui::ui_touch::UITouch;
use winit::event::KeyboardInput;

pub trait UIObject: GameObjectBase {
    fn z_index(&self) -> i32 {
        0
    }

    fn on_key_up(&mut self, input: &KeyboardInput);

    fn on_key_down(&mut self, input: &KeyboardInput);

    fn on_touch_start(&mut self, index: u64, touch: &UITouch);

    fn on_touch_move(&mut self, index: u64, touch: &UITouch);

    fn on_touch_end(&mut self, index: u64, touch: &UITouch);

    fn on_touch_cancel(&mut self, index: u64, touch: &UITouch);
}
