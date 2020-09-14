use gfx_hal::adapter::Adapter;
use gfx_hal::{Backend, Instance};
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_backend<B: Backend>(
    window: &Window,
) -> (Option<B::Instance>, Vec<Adapter<B>>, B::Surface) {
    let instance = B::Instance::create("gfx-rs quad", 1).expect("Failed to create an instance!");
    let adapters = instance.enumerate_adapters();
    let surface = unsafe {
        instance
            .create_surface(window)
            .expect("Failed to create a surface!")
    };

    // Return `window` so it is not dropped: dropping it invalidates `surface`.
    (Some(instance), adapters, surface)
}

#[cfg(target_arch = "wasm32")]
pub fn create_backend(
    window: &Window,
) -> (
    Option<gfx_backend_gl::Surface>,
    Vec<Adapter<gfx_backend_gl::Backend>>,
    gfx_backend_gl::Surface,
) {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap()
        .append_child(&WindowExtWebSys::canvas(window))
        .unwrap();

    let surface = gfx_backend_gl::Surface::from_raw_handle(window);
    let adapters = surface.enumerate_adapters();
    (None, adapters, surface)
}
