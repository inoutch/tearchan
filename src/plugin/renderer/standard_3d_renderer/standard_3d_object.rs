use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;

pub trait Standard3DObject: GameObjectBase {
    fn attach_queue(&mut self, queue: BatchCommandQueue);
}
