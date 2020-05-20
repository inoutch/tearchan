use crate::core::engine_config::StartupConfig;
use crate::core::graphic::hal::instance::create_backend;
use crate::core::graphic::hal::renderer::Renderer;
use crate::core::scene::scene_manager::SceneManager;
use crate::core::screen::ScreenMode;
use std::time::Instant;
use winit::event_loop::EventLoop;
use winit::monitor::MonitorHandle;

pub struct Engine {
    startup_config: StartupConfig,
    scene_manager: SceneManager,
}

impl Engine {
    pub fn new(mut config: StartupConfig) -> Engine {
        let scene_creator =
            std::mem::replace(&mut config.scene_creator, None).expect("specify a scene creator");
        Engine {
            startup_config: config,
            scene_manager: SceneManager::new(scene_creator),
        }
    }

    pub fn run(self) {
        let window_size = match &self.startup_config.screen_mode {
            ScreenMode::FullScreenWindow => unimplemented!("FullScreenWindow"),
            ScreenMode::Windowed { resolutions } => &resolutions[0],
        };

        let event_loop = winit::event_loop::EventLoop::new();
        let window_builder = winit::window::WindowBuilder::new()
            .with_min_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                64.0, 64.0,
            )))
            .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                window_size.width as f64,
                window_size.height as f64,
            )))
            /*.with_fullscreen(Some(Fullscreen::Borderless(prompt_for_monitor(
                &event_loop,
            ))))*/
            .with_title(self.startup_config.application_name.to_string());

        let (window, instance, mut adapters, surface) = create_backend(window_builder, &event_loop);
        let adapter = adapters.remove(0);
        let mut renderer = Renderer::new(instance, adapter, surface, *window_size);
        let mut scene_manager = self.scene_manager;

        let timer_length = std::time::Duration::from_millis(1000 / 60);
        event_loop.run(move |event, _, control_flow| match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = winit::event_loop::ControlFlow::Exit,
                _ => {}
            },
            winit::event::Event::RedrawRequested(_) => {
                renderer.render(|api| {
                    scene_manager.render(1.0f32 / 6.0f32, api);
                });
            },
            _ => {
                *control_flow =
                    winit::event_loop::ControlFlow::WaitUntil(Instant::now() + timer_length);
                window.request_redraw();
            }
        });
    }
}

fn prompt_for_monitor(event_loop: &EventLoop<()>) -> MonitorHandle {
    let num = 0;
    let monitor = event_loop
        .available_monitors()
        .nth(num)
        .expect("Please enter a valid ID");

    println!("Using {:?}", monitor.name());
    monitor
}
