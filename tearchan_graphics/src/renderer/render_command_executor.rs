use crate::renderer::render_command::RenderCommand;
use crate::renderer::render_command_queue::RenderCommandQueue;
use crate::renderer::{RenderId, Renderer};
use std::collections::HashMap;
use std::option::Option::Some;
use std::sync::{Arc, Mutex};
use tearchan_utility::id_manager::IdManager;

pub type RendererType = u32;

pub struct RenderCommandExecutor {
    commands: Arc<Mutex<Vec<RenderCommand>>>,
    renderers: HashMap<RendererType, Box<dyn Renderer>>,
    id_manager: IdManager<RenderId>,
    id_to_renderer_map: HashMap<RenderId, RendererType>,
}

impl RenderCommandExecutor {
    pub fn new() -> Self {
        RenderCommandExecutor {
            commands: Arc::new(Mutex::new(vec![])),
            renderers: HashMap::new(),
            id_manager: IdManager::new(0u64, |id| id + 1u64),
            id_to_renderer_map: HashMap::new(),
        }
    }

    pub fn render(&mut self) {
        loop {
            let command = match self.commands.lock().unwrap().pop() {
                Some(command) => command,
                None => break,
            };

            match &command {
                RenderCommand::Add {
                    id,
                    renderer_type,
                    vertices,
                    order,
                } => {
                    debug_assert!(
                        self.renderers.contains_key(renderer_type),
                        "renderer_type is not registered: {}",
                        renderer_type
                    );

                    self.id_to_renderer_map.insert(*id, *renderer_type);
                    self.renderers
                        .get_mut(&renderer_type)
                        .map(|r| r.add(*id, &vertices, order.unwrap_or_else(|| std::i32::MAX)));
                }
                RenderCommand::Remove { id } => {
                    let renderer_type = self.id_to_renderer_map.remove(id);
                    debug_assert!(renderer_type.is_some(), "id is not registered: {}", id);

                    if let Some(renderer_type) = renderer_type {
                        self.renderers
                            .get_mut(&renderer_type)
                            .map(|r| r.remove(*id));
                    }
                }
                RenderCommand::Transform { .. } => {}
                RenderCommand::Copy { .. } => {}
                RenderCommand::CopyForEach { .. } => {}
            }
        }
    }

    pub fn create_queue(&mut self) -> RenderCommandQueue {
        RenderCommandQueue::new(
            Arc::clone(&self.commands),
            self.id_manager.create_generator(),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::renderer::render_command_executor::RenderCommandExecutor;

    #[test]
    fn test() {
        let mut executor = RenderCommandExecutor::new();
        executor.render();
    }
}
