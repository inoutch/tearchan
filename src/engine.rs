use crate::engine_config::{EngineConfig, StartupConfig};
use nalgebra_glm::vec2;
use tearchan_graphics::hal::backend::create_fixed_backend;
use tearchan_graphics::hal::renderer::Renderer;
use tearchan_graphics::screen::ScreenMode;
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoop;
use winit::monitor::MonitorHandle;
use winit::window::{Fullscreen, WindowBuilder};

pub struct Engine {
    config: EngineConfig,
}

impl Engine {
    pub fn new(config: StartupConfig) -> Engine {
        Engine {
            config: EngineConfig {
                application_name: config.application_name,
                screen_mode: config.screen_mode,
                screen_size: config.screen_size,
                screen_resolution_mode: config.screen_resolution_mode,
                min_physical_size: config.min_physical_size,
                resource_path: config.resource_path,
                writable_path: config.writable_path,
                fps: config.fps,
                renderer_properties: config.renderer_properties,
            },
            // scene_manager: SceneManager::new(config.scene_creator),
        }
    }

    pub fn run(self) {
        let event_loop = EventLoop::new();
        let monitor = prompt_for_monitor(&event_loop);
        let physical_size = match &self.config.screen_mode {
            ScreenMode::FullScreenWindow => vec2(monitor.size().width, monitor.size().height),
            ScreenMode::Windowed { resolutions } => resolutions[0].clone_owned(),
        };

        let mut window_builder = WindowBuilder::new()
            .with_title(self.config.application_name)
            .with_min_inner_size(Size::Physical(PhysicalSize::new(
                self.config.min_physical_size.x,
                self.config.min_physical_size.y,
            )));

        window_builder = match &self.config.screen_mode {
            ScreenMode::FullScreenWindow => {
                window_builder.with_fullscreen(Some(Fullscreen::Borderless(monitor)))
            }
            ScreenMode::Windowed { resolutions } => {
                assert!(
                    !resolutions.is_empty(),
                    "In fullscreen mode, it must have at least one resolution"
                );
                window_builder.with_inner_size(Size::Physical(PhysicalSize::new(
                    resolutions[0].x,
                    resolutions[0].y,
                )))
            }
        };

        let window = window_builder.build(&event_loop).unwrap();
        let (instance, mut adapters, surface) = create_fixed_backend(&window);
        let mut renderer = Renderer::new(
            instance,
            adapters.remove(0),
            surface,
            physical_size,
            self.config.renderer_properties,
        );
        renderer.set_screen_resolution_mode(&self.config.screen_resolution_mode);
    }
}

pub fn prompt_for_monitor(event_loop: &EventLoop<()>) -> MonitorHandle {
    let num = 0;
    event_loop.available_monitors().nth(num).unwrap()
}
