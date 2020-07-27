use crate::core::engine_config::{EngineConfig, StartupConfig};
use crate::core::file::File;
use crate::core::graphic::hal::instance::create_backend;
use crate::core::graphic::hal::renderer::Renderer;
use crate::core::scene::scene_manager::SceneManager;
use crate::core::screen::ScreenMode;
use gfx_hal::window::Extent2D;
use std::cell::RefCell;
use std::time::Instant;
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoop;
use winit::monitor::MonitorHandle;
use winit::window::Fullscreen;

const WINDOW_MIN_SIZE_WIDTH: f64 = 64.0f64;
const WINDOW_MIN_SIZE_HEIGHT: f64 = 64.0f64;

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
                screen_resolution_mode: config.screen_resolution_mode,
                resource_path: config.resource_path,
                writable_path: config.writable_path,
                fps: config.fps,
            },
            scene_manager: SceneManager::new(config.scene_creator),
        }
    }

    pub fn run(self) {
        let event_loop = winit::event_loop::EventLoop::new();
        let monitor = prompt_for_monitor(&event_loop);

        let window_size = match &self.config.screen_mode {
            ScreenMode::FullScreenWindow => Extent2D {
                width: monitor.size().width,
                height: monitor.size().height,
            },
            ScreenMode::Windowed { resolutions } => resolutions[0],
        };

        let mut window_builder = winit::window::WindowBuilder::new()
            .with_min_inner_size(Size::Logical(LogicalSize::new(
                WINDOW_MIN_SIZE_WIDTH,
                WINDOW_MIN_SIZE_HEIGHT,
            )))
            .with_title(self.config.application_name.to_string());
        window_builder = match &self.config.screen_mode {
            ScreenMode::FullScreenWindow => {
                window_builder.with_fullscreen(Some(Fullscreen::Borderless(monitor)))
            }
            ScreenMode::Windowed { resolutions } => {
                assert!(
                    !resolutions.is_empty(),
                    "In fullscreen mode, it must have at least one resolution"
                );
                window_builder.with_inner_size(winit::dpi::Size::Physical(
                    winit::dpi::PhysicalSize::new(resolutions[0].width, resolutions[0].height),
                ))
            }
        };

        let screen_resolution_mode = self.config.screen_resolution_mode.clone();
        let (window, instance, mut adapters, surface) = create_backend(window_builder, &event_loop);
        let adapter = adapters.remove(0);
        let mut renderer = Renderer::new(instance, adapter, surface, window_size);
        renderer.display_size_mut().update(&screen_resolution_mode);

        let mut file = File::new(self.config.resource_path, self.config.writable_path);
        let scene_manager = RefCell::new(self.scene_manager);

        let mut prev_time = std::time::Instant::now();
        let timer_length = std::time::Duration::from_millis(1000 / self.config.fps);
        event_loop.run(move |event, _, control_flow| match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                winit::event::WindowEvent::Resized(dims) => {
                    renderer.set_dimensions(Extent2D {
                        width: dims.width,
                        height: dims.height,
                    });
                    renderer.recreate_swapchain();
                    renderer.display_size_mut().update(&screen_resolution_mode);
                    let mut context = renderer.create_resize_context();
                    scene_manager.borrow_mut().resize(&mut context);
                }
                _ => {
                    scene_manager.borrow_mut().event(&event);
                }
            },
            winit::event::Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                renderer.render(
                    |renderer_api| {
                        scene_manager.borrow_mut().render(
                            (now - prev_time).as_secs_f32(),
                            renderer_api,
                            &mut file,
                        );
                    },
                    |context| {
                        scene_manager.borrow_mut().resize(context);
                    },
                );
                prev_time = now;
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
