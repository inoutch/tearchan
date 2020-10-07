use crate::plugin::renderer::billboard_renderer::billboard_command_queue::BillboardCommandQueue;
use tearchan_core::game::object::game_object_base::GameObjectBase;

pub trait BillboardRenderObject: GameObjectBase {
    fn attach_queue(&mut self, queue: BillboardCommandQueue);

    fn detach(&mut self);
}
