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
#[cfg(any(feature = "wgl"))]
extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;

use crate::hal::instance::create_backend;
use crate::hal::render_bundle::RenderBundleCommon;
use crate::hal::renderer_context::RendererContextCommon;
use gfx_hal::adapter::Adapter;
use winit::window::Window;

pub type Backend = back::Backend;
pub type Surface = back::Surface;

pub type RenderBundle = RenderBundleCommon<Backend>;
pub type RendererContext<'a> = RendererContextCommon<'a, Backend>;

#[cfg(not(target_arch = "wasm32"))]
pub type Instance = back::Instance;

#[cfg(target_arch = "wasm32")]
pub type Instance = back::Surface;

pub fn create_fixed_backend(window: &Window) -> (Option<Instance>, Vec<Adapter<Backend>>, Surface) {
    create_backend(window)
}
