use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;

pub trait StandardLineRenderObject: GameObjectBase {
    fn attach_queue(&mut self, queue: BatchCommandQueue);

    fn detach(&mut self);
}
