use crate::controller::touch::Touch;
use intertrait::CastFrom;
use winit::event::KeyboardInput;

pub trait UIObject: CastFrom {
    fn z_index(&self) -> i32 {
        0
    }

    fn on_key_up(&mut self, input: &KeyboardInput);

    fn on_key_down(&mut self, input: &KeyboardInput);

    fn on_touch_start(&mut self, index: u64, touch: &Touch);

    fn on_touch_move(&mut self, index: u64, touch: &Touch);

    fn on_touch_end(&mut self, index: u64, touch: &Touch);

    fn on_touch_cancel(&mut self, index: u64, touch: &Touch);
}
