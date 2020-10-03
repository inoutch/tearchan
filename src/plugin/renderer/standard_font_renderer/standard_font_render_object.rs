use crate::plugin::renderer::standard_font_renderer::standard_font_command_queue::StandardFontCommandQueue;
use tearchan_core::game::object::game_object_base::GameObjectBase;

pub trait StandardFontRenderObject: GameObjectBase {
    fn attach_queue(&mut self, queue: StandardFontCommandQueue);
}
