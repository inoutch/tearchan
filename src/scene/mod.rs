use crate::scene::context::{SceneContext, SceneRenderContext};
use crate::scene::factory::{SceneFactory, SceneOption};
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;

pub mod context;
pub mod factory;
pub mod manager;

pub trait Scene {
    fn update(&mut self, context: &mut SceneContext, event: WindowEvent) -> SceneControlFlow;
    fn render(&mut self, context: &mut SceneRenderContext) -> SceneControlFlow;
}

pub enum SceneControlFlow {
    None,
    Winit {
        control_flow: ControlFlow,
    },
    TransitScene {
        factory: SceneFactory,
        option: Option<Box<dyn SceneOption>>,
    },
}
