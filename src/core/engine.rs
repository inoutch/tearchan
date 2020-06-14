use crate::core::engine_config::{EngineConfig, StartupConfig};
use crate::core::file::file_api::FileApi;
use crate::core::graphic::hal::instance::create_backend;
use crate::core::graphic::hal::renderer::Renderer;
use crate::core::scene::scene_manager::SceneManager;
use crate::core::screen::ScreenMode;
use gfx_hal::window::Extent2D;
use std::time::Instant;
use winit::event_loop::EventLoop;
use winit::monitor::MonitorHandle;
use winit::window::Fullscreen;

pub struct Engine {
    config: EngineConfig,
    scene_manager: SceneManager,
}

impl Engine {
    pub fn new(config: StartupConfig) -> Engine {
        Engine {
            config: EngineConfig {
                application_name: config.application_name,
                screen_mode: config.screen_mode,
                screen_size: config.screen_size,
                resource_path: config.resource_path,
                writable_path: config.writable_path,
            },
            scene_manager: SceneManager::new(config.scene_creator),
        }
    }

    pub fn run(self) {
        let window_size = match &self.config.screen_mode {
            ScreenMode::FullScreenWindow => Extent2D {
                width: 1,
                height: 1,
            },
            ScreenMode::Windowed { resolutions } => resolutions[0],
        };

        let event_loop = winit::event_loop::EventLoop::new();
        let mut window_builder = winit::window::WindowBuilder::new()
            .with_min_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                64.0, 64.0,
            )))
            .with_title(self.config.application_name.to_string());
        window_builder = match &self.config.screen_mode {
            ScreenMode::FullScreenWindow => window_builder.with_fullscreen(Some(
                Fullscreen::Borderless(prompt_for_monitor(&event_loop)),
            )),
            ScreenMode::Windowed { resolutions } => window_builder.with_inner_size(
                winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                    resolutions[0].width as f64,
                    resolutions[0].height as f64,
                )),
            ),
        };

        let (window, instance, mut adapters, surface) = create_backend(window_builder, &event_loop);
        let adapter = adapters.remove(0);
        let mut renderer = Renderer::new(instance, adapter, surface, window_size);
        let mut scene_manager = self.scene_manager;
        let mut file_api = FileApi::new(self.config.resource_path, self.config.writable_path);

        let timer_length = std::time::Duration::from_millis(1000 / 60);
        event_loop.run(move |event, _, control_flow| match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                _ => {
                    scene_manager.event(&event);
                }
            },
            winit::event::Event::RedrawRequested(_) => {
                renderer.render(|renderer_api| {
                    scene_manager.render(1.0f32 / 6.0f32, renderer_api, &mut file_api);
                });
            }
            winit::event::Event::MainEventsCleared => {
                *control_flow =
                    winit::event_loop::ControlFlow::WaitUntil(Instant::now() + timer_length);
                window.request_redraw();
            }
            _ => {}
        });
    }
}

pub fn prompt_for_monitor(event_loop: &EventLoop<()>) -> MonitorHandle {
    let num = 0;
    event_loop
        .available_monitors()
        .nth(num)
        .expect("Please enter a valid ID")
}
