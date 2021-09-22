use crate::scene::context::{SceneContext, SceneRenderContext};
use crate::scene::factory::{SceneFactory, SceneOption};
use crate::scene::{Scene, SceneControlFlow};
use tearchan_gfx::wgpu::Maintain;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;

pub struct SceneManager {
    current_scene: Box<dyn Scene>,
    current_scene_factory: Option<(SceneFactory, Option<Box<dyn SceneOption>>)>,
}

impl Default for SceneManager {
    fn default() -> Self {
        SceneManager {
            current_scene: Box::new(DummyScene),
            current_scene_factory: None,
        }
    }
}

impl SceneManager {
    pub fn update(&mut self, event: WindowEvent, mut context: SceneContext) -> Option<ControlFlow> {
        let control_flow = self.current_scene.update(&mut context, event);
        self.process(control_flow, &context)
    }

    pub fn render(&mut self, context: &mut SceneRenderContext) -> Option<ControlFlow> {
        self.recreate_scene(context);

        let control_flow = self.current_scene.render(context);
        self.process(control_flow, &context)
    }

    pub fn set_current_scene(&mut self, scene: SceneFactory, option: Option<Box<dyn SceneOption>>) {
        self.current_scene_factory = Some((scene, option));
    }

    fn recreate_scene(&mut self, context: &mut SceneContext) {
        if let Some((scene_factory, options)) =
            std::mem::replace(&mut self.current_scene_factory, None)
        {
            self.current_scene = scene_factory(context, options);
        }
    }

    fn process(
        &mut self,
        control_flow: SceneControlFlow,
        context: &SceneContext,
    ) -> Option<ControlFlow> {
        match control_flow {
            SceneControlFlow::None => {}
            SceneControlFlow::Winit { control_flow } => return Some(control_flow),
            SceneControlFlow::TransitScene { factory, option } => {
                context.gfx().device.poll(Maintain::Wait);
                self.current_scene_factory = Some((factory, option));
            }
        }
        None
    }
}

struct DummyScene;

impl Scene for DummyScene {
    fn update(&mut self, _context: &mut SceneContext, _event: WindowEvent) -> SceneControlFlow {
        SceneControlFlow::None
    }

    fn render(&mut self, _context: &mut SceneRenderContext) -> SceneControlFlow {
        SceneControlFlow::None
    }
}
