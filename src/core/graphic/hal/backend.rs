#[cfg(feature = "dx11")]
extern crate gfx_backend_dx11 as back;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(not(any(
    feature = "vulkan",
    feature = "dx11",
    feature = "dx12",
    feature = "metal",
    feature = "gl",
    feature = "wgl"
)))]
extern crate gfx_backend_empty as back;
#[cfg(any(feature = "gl"))]
extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;

pub type FixedBackend = back::Backend;
pub type FixedInstance = back::Instance;
pub type FixedSurface = back::Surface;

pub type FixedApi<'a> = Api<'a, back::Backend>;
pub type FixedVertexBuffer = VertexBuffer<back::Backend>;
pub type FixedUniformBuffer<T> = UniformBuffer<back::Backend, T>;
pub type FixedTexture = Texture<back::Backend>;
pub type FixedGraphicPipeline = GraphicPipeline<back::Backend>;

use gfx_hal::adapter::Adapter;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::core::graphic::hal::graphic_pipeline::GraphicPipeline;
use crate::core::graphic::hal::renderer_api::Api;
use crate::core::graphic::hal::texture::Texture;
use crate::core::graphic::hal::uniform_buffer::UniformBuffer;
use crate::core::graphic::hal::vertex_buffer::VertexBuffer;
use gfx_hal::Instance;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_backend<T: 'static>(
    wb: WindowBuilder,
    event_loop: &EventLoopWindowTarget<T>,
) -> (
    Window,
    Option<FixedInstance>,
    Vec<Adapter<FixedBackend>>,
    FixedSurface,
) {
    let window = wb.build(&event_loop).unwrap();

    let instance = FixedInstance::create("gfx-rs quad", 1).expect("Failed to create an instance!");
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
    Option<FixedInstance>,
    Vec<Adapter<FixedBackend>>,
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
