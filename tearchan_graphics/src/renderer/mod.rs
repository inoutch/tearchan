use crate::renderer::render_command::{
    RenderCommandTransform, RenderCommandValue, RenderCommandVertices,
};

pub mod render_command;
pub mod render_command_executor;
pub mod render_command_queue;

pub type RenderId = u64;
pub const RENDER_ID_EMPTY: u64 = std::u64::MAX;

pub trait Renderer {
    fn add(&mut self, id: RenderId, vertices: &Vec<RenderCommandVertices>, order: i32);

    fn remove(&mut self, id: RenderId);

    fn transform(&mut self, id: RenderId, attribute: u32, transform: RenderCommandTransform);

    fn copy_all(&mut self, id: RenderId, value: RenderCommandValue);
}
