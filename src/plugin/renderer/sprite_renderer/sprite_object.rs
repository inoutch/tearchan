use crate::plugin::renderer::sprite_renderer::sprite_command_queue::SpriteCommandQueue;
use tearchan_core::game::object::game_object_base::GameObjectBase;

pub trait SpriteObject: GameObjectBase {
    fn attach_sprite_queue(&mut self, queue: SpriteCommandQueue);
}

#[cfg(test)]
mod test {
    use crate::plugin::renderer::sprite_renderer::sprite_command_queue::SpriteCommandQueue;
    use crate::plugin::renderer::sprite_renderer::sprite_object::SpriteObject;
    use std::sync::mpsc::channel;
    use tearchan_core::game::object::game_object_base::GameObjectBase;
    use tearchan_graphics::batch::batch_command_queue::BatchCommandQueue;
    use tearchan_utility::id_manager::IdManager;

    struct ExampleSprite {
        queue: Option<SpriteCommandQueue>,
    }

    impl GameObjectBase for ExampleSprite {}

    impl SpriteObject for ExampleSprite {
        fn attach_sprite_queue(&mut self, queue: SpriteCommandQueue) {
            self.queue = Some(queue)
        }
    }

    #[test]
    fn test() {
        let mut sprite = ExampleSprite { queue: None };
        let (sender, _receiver) = channel();
        let id_manager = IdManager::new(0, |id| id + 1);
        sprite.attach_sprite_queue(SpriteCommandQueue::new(BatchCommandQueue::new(
            sender,
            id_manager.create_generator(),
        )));

        assert!(sprite.queue.is_some());
    }
}
