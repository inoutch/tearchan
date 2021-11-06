use crate::engine_config::EngineStartupConfig;
use crate::scene::context::{SceneContext, SceneRenderContext};
use crate::scene::manager::SceneManager;
use instant::Instant;
use std::future::Future;
use std::time::Duration;
use tearchan_gfx::renderer::RendererLazySetup;
use tearchan_util::any::OptAnyBox;
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
        let mut custom = OptAnyBox::new(startup_config.custom);
        let spawner = Spawner::default();

        let event_loop = EventLoop::new();
        let window = startup_config.window_builder.build(&event_loop).unwrap();
        let mut scale_factor = window.scale_factor();

        let mut setup = RendererLazySetup::new(window);
        if !cfg!(target_os = "android") {
            setup
                .setup(
                    tearchan_gfx::wgpu::Features::empty(),
                    tearchan_gfx::wgpu::Features::empty(),
                    tearchan_gfx::wgpu::Limits::default(),
                )
                .await;
        }

        let mut scene_manager = SceneManager::default();
        scene_manager.set_current_scene(startup_config.scene_factory, None);

        let target_frametime = Duration::from_secs_f64(1.0 / startup_config.fps as f64);
        let mut last_update_inst = Instant::now();
        let mut last_frame_inst = Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            #[cfg(target_os = "android")]
            Event::Resumed => {
                let executor = async_executor::LocalExecutor::new();
                executor
                    .spawn(setup.setup(
                        wgpu::Features::empty(),
                        wgpu::Features::empty(),
                        wgpu::Limits::default(),
                    ))
                    .detach();
                while executor.try_tick() {}
            }
            Event::WindowEvent { event, window_id } => {
                match &event {
                    WindowEvent::Resized(size) => {
                        if let Some(renderer) = setup.renderer_mut() {
                            renderer.resize(*size);
                        }
                    }
                    WindowEvent::CloseRequested => {
                        if window_id == setup.window().id() {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    WindowEvent::ScaleFactorChanged {
                        scale_factor: new_scale_factor,
                        ..
                    } => {
                        scale_factor = *new_scale_factor;
                    }
                    _ => {}
                }
                if let Some(renderer) = setup.renderer_mut() {
                    let context = SceneContext::new(
                        renderer.create_context(),
                        &spawner,
                        &mut custom,
                        scale_factor,
                    );
                    if let Some(overwrite) = scene_manager.update(event, context) {
                        *control_flow = overwrite;
                    };
                }
            }
            Event::RedrawEventsCleared => {
                let time_since_last_frame = last_update_inst.elapsed();
                if time_since_last_frame >= target_frametime {
                    setup.window().request_redraw();
                    last_update_inst = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now() + target_frametime - time_since_last_frame,
                    );
                }
                spawner.run_until_stalled();
            }
            Event::RedrawRequested(_) => {
                let delta = last_frame_inst.elapsed().as_secs_f32();
                last_frame_inst = Instant::now();

                // Rendering
                if let Some(renderer) = setup.renderer_mut() {
                    let frame = renderer.create_surface_texture();
                    let (context, render_context) = renderer.create_render_context(&frame);
                    let mut context = SceneRenderContext::new(
                        (context, render_context),
                        &spawner,
                        &mut custom,
                        scale_factor,
                        delta,
                    );
                    if let Some(overwrite) = scene_manager.render(&mut context) {
                        *control_flow = overwrite;
                    }

                    frame.present();
                }
            }
            _ => (),
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<'a> Spawner<'a> {
    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'a) {
        self.executor.spawn(future).detach();
    }

    fn run_until_stalled(&self) {
        while self.executor.try_tick() {}
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub struct Spawner<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

#[cfg(target_arch = "wasm32")]
impl<'a> Spawner<'a> {
    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }
}

// HACK: https://github.com/rust-windowing/winit/pull/1822
#[cfg(target_os = "android")]
fn request_redraw_for_android() {
    match ndk_glue::native_window().as_ref() {
        Some(native_window) => {
            let a_native_window: *mut ndk_sys::ANativeWindow = native_window.ptr().as_ptr();
            let a_native_activity: *mut ndk_sys::ANativeActivity =
                ndk_glue::native_activity().ptr().as_ptr();
            unsafe {
                match (*(*a_native_activity).callbacks).onNativeWindowRedrawNeeded {
                    Some(callback) => callback(a_native_activity, a_native_window),
                    None => (),
                };
            };
        }
        None => (),
    }
}
