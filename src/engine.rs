use crate::engine_config::EngineStartupConfig;
use crate::scene::context::{SceneContext, SceneRenderContext};
use crate::scene::manager::SceneManager;
use instant::Instant;
use std::time::Duration;
use tearchan_gfx::setup::Setup;
use tearchan_util::time::DurationWatch;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

pub struct Engine {
    startup_config: EngineStartupConfig,
}

impl Engine {
    pub fn new(startup_config: EngineStartupConfig) -> Self {
        Engine { startup_config }
    }

    pub fn run(self) {
        tearchan_core::sync::run(self.start());
    }

    async fn start(self) {
        let startup_config = self.startup_config;

        let event_loop = EventLoop::new();
        let window = startup_config.window_builder.build(&event_loop).unwrap();
        let mut setup = Setup::new(window);
        let mut scene_manager = SceneManager::default();
        scene_manager.set_current_scene(startup_config.scene_factory, None);

        let duration = Duration::from_millis(1000 / startup_config.fps).as_millis() as u64;
        let mut start_time = Instant::now();
        let mut duration_watcher = DurationWatch::default();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::Resized(_) => {}
                WindowEvent::CloseRequested => {
                    if window_id == setup.window().id() {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => {
                    let context = SceneContext::new(setup.create_context());
                    if let Some(overwrite) = scene_manager.update(event, context) {
                        *control_flow = overwrite;
                    };
                }
            },
            Event::MainEventsCleared => {
                start_time = Instant::now();
                setup.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let elapsed_time = Instant::now().duration_since(start_time).as_millis() as u64;
                let wait_millis = match duration >= elapsed_time {
                    true => duration - elapsed_time,
                    false => 0,
                };
                let new_inst = start_time + std::time::Duration::from_millis(wait_millis);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    *control_flow = ControlFlow::WaitUntil(new_inst);
                }
                #[cfg(target_arch = "wasm32")]
                {
                    *control_flow = ControlFlow::Poll;
                }
                // Rendering
                let context = SceneRenderContext::new(setup.create_render_context());
                if let Some(overwrite) = scene_manager.render(context) {
                    *control_flow = overwrite;
                };

                setup.flush();
                duration_watcher.reset();
            }
            _ => (),
        });
    }
}
