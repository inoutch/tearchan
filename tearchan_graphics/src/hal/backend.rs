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

use crate::hal::buffer::index_buffer::IndexBufferCommon;
use crate::hal::buffer::uniform_buffer::UniformBufferCommon;
use crate::hal::buffer::vertex_buffer::VertexBufferCommon;
use crate::hal::font_texture::FontTextureCommon;
use crate::hal::graphic_pipeline::GraphicPipelineCommon;
use crate::hal::instance::create_backend;
use crate::hal::render_bundle::RenderBundleCommon;
use crate::hal::renderer_context::RendererContextCommon;
use crate::hal::shader::descriptor_set::DescriptorSetCommon;
use crate::hal::shader::write_descriptor_sets::WriteDescriptorSetsCommon;
use crate::hal::shader::ShaderCommon;
use crate::hal::texture::TextureCommon;
use gfx_hal::adapter::Adapter;
use winit::window::Window;

pub type Backend = back::Backend;
pub type Surface = back::Surface;

pub type RenderBundle = RenderBundleCommon<Backend>;
pub type RendererContext<'a> = RendererContextCommon<'a, Backend>;

pub type Shader = ShaderCommon<Backend>;
pub type DescriptorSet = DescriptorSetCommon<Backend>;
pub type WriteDescriptorSets<'a> = WriteDescriptorSetsCommon<'a, Backend>;
pub type UniformBuffer<T> = UniformBufferCommon<Backend, T>;
pub type IndexBuffer = IndexBufferCommon<Backend>;
pub type VertexBuffer = VertexBufferCommon<Backend>;
pub type Texture = TextureCommon<Backend>;
pub type FontTexture = FontTextureCommon<Backend>;
pub type GraphicPipeline = GraphicPipelineCommon<Backend>;

#[cfg(not(target_arch = "wasm32"))]
pub type Instance = back::Instance;

#[cfg(target_arch = "wasm32")]
pub type Instance = back::Surface;

pub fn create_fixed_backend(window: &Window) -> (Option<Instance>, Vec<Adapter<Backend>>, Surface) {
    create_backend(window)
}
