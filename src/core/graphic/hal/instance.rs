use crate::core::graphic;
use crate::core::graphic::hal::backend::{Backend, Surface};
use gfx_hal::adapter::Adapter;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use gfx_hal::Instance;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_backend<T: 'static>(
    wb: WindowBuilder,
    event_loop: &EventLoopWindowTarget<T>,
) -> (
    Window,
    Option<graphic::hal::backend::Instance>,
    Vec<Adapter<Backend>>,
    Surface,
) {
    let window = wb.build(&event_loop).unwrap();

    let instance = graphic::hal::backend::Instance::create("gfx-rs quad", 1)
        .expect("Failed to create an instance!");
    let adapters = instance.enumerate_adapters();
    let surface = unsafe {
        instance
            .create_surface(&window)
            .expect("Failed to create a surface!")
    };

    // Return `window` so it is not dropped: dropping it invalidates `surface`.
    (window, Some(instance), adapters, surface)
}

#[cfg(target_arch = "wasm32")]
pub fn create_backend<T: 'static>(
    wb: WindowBuilder,
    event_loop: &EventLoopWindowTarget<T>,
) -> (
    Window,
    Option<Instance>,
    Vec<Adapter<Backend>>,
    FixedSurface,
) {
    let (window, surface) = {
        let window = wb.build(&event_loop).unwrap();
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&winit::platform::web::WindowExtWebSys::canvas(&window))
            .unwrap();
        let surface = B::Surface::from_raw_handle(&window);
        (window, surface)
    };

    let adapters = surface.enumerate_adapters();
    (window, None, adapters, surface)
}
