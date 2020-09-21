use crate::game::game_context::GameContext;
use crate::game::object::game_object_base::GameObjectBase;
use crate::game::object::GameObject;
use tearchan_graphics::display_size::DisplaySize;
use winit::event::WindowEvent;

pub trait GamePlugin {
    fn on_add(&mut self, _game_object: &GameObject<dyn GameObjectBase>) {}

    fn on_remove(&mut self, _game_object: &GameObject<dyn GameObjectBase>) {}

    fn on_update(&mut self, _context: &mut GameContext) {}

    fn on_window_event(&mut self, _event: &WindowEvent) {}

    fn on_resize(&mut self, _display_size: &DisplaySize) {}
}
