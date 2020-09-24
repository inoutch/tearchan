use crate::engine_config::{EngineConfig, StartupConfig};
use nalgebra_glm::vec2;
use std::ops::Deref;
use std::time::{Duration, Instant};
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::game_plugin_manager::GamePluginManager;
use tearchan_core::scene::scene_manager::SceneManager;
use tearchan_core::ui::ui_manager::UIManager;
use tearchan_graphics::hal::backend::create_fixed_backend;
use tearchan_graphics::hal::renderer::{Renderer, RendererBeginResult};
use tearchan_graphics::screen::ScreenMode;
use tearchan_utility::time::DurationWatch;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::monitor::MonitorHandle;
use winit::window::{Fullscreen, WindowBuilder};

pub struct Engine {
    config: EngineConfig,
    plugin_manager: GamePluginManager,
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
                scene_factory: config.scene_factory,
                resource_path: config.resource_path,
                writable_path: config.writable_path,
                fps: config.fps,
                renderer_properties: config.renderer_properties,
            },
            plugin_manager: GamePluginManager::new(),
        }
    }

    pub fn with_plugin(&mut self, plugin: Box<dyn GamePlugin>, key: String, order: i32) {
        self.plugin_manager.add(plugin, key, order);
    }

    pub fn with_default_plugins(mut self) -> Self {
        self.with_plugin(Box::new(UIManager::new()), "UIManager".to_string(), 0);
        self
    }

    pub fn run(self) {
        let event_loop = EventLoop::new();
        let monitor = prompt_for_monitor(&event_loop);
        let screen_resolution_mode = self.config.screen_resolution_mode;
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

        // Prepare renderer
        let window = window_builder.build(&event_loop).unwrap();
        let (instance, mut adapters, surface) = create_fixed_backend(&window);
        let mut renderer = Renderer::new(
            instance,
            adapters.remove(0),
            surface,
            physical_size,
            self.config.renderer_properties,
        );
        renderer.set_screen_resolution_mode(&screen_resolution_mode);

        let mut plugin_manager = self.plugin_manager;
        // TODO: Prepare file manager
        // TODO: Prepare sound manager
        // TODO: Prepare network manager

        // TODO: Prepare asset manager
        // TODO: Prepare touch manager
        // TODO: Prepare object manager

        let mut scene_manager = SceneManager::new(self.config.scene_factory);
        let mut duration_watch = DurationWatch::default();
        let duration = Duration::from_millis(1000 / self.config.fps);

        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {}
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Resized(size) => {
                        let v_size = vec2(size.width as _, size.height as _);
                        renderer.set_dimensions(v_size);
                        renderer.recreate_swapchain();
                        renderer.set_screen_resolution_mode(&screen_resolution_mode);
                        plugin_manager.for_each_mut(|plugin| {
                            plugin.on_resize(renderer.display_size().deref());
                        });
                    }
                    _ => {
                        plugin_manager.for_each_mut(|plugin| {
                            plugin.on_window_event(&event);
                        });
                    }
                };
            }
            Event::DeviceEvent { .. } => {}
            Event::UserEvent(_) => {}
            Event::Suspended => {}
            Event::Resumed => {}
            Event::MainEventsCleared => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + duration);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let delta = duration_watch.measure_as_sec();
                renderer.render(|event| match event {
                    RendererBeginResult::Context { context } => {
                        let mut game_context = GameContext::new(delta, context);
                        scene_manager.on_update(&mut game_context, &mut plugin_manager);

                        plugin_manager.for_each_mut(|plugin| {
                            plugin.on_update(&mut game_context);
                        });
                    }
                    RendererBeginResult::Resize => {}
                });
                duration_watch.reset();
            }
            Event::RedrawEventsCleared => {}
            Event::LoopDestroyed => {}
        });
    }
}

pub fn prompt_for_monitor(event_loop: &EventLoop<()>) -> MonitorHandle {
    // TODO: Manage monitor with config
    let num = 0;
    event_loop.available_monitors().nth(num).unwrap()
}
