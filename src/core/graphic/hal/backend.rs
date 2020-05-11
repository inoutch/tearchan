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

use crate::core::graphic::hal::graphic_pipeline::GraphicPipelineCommon;
use crate::core::graphic::hal::renderer_api::RendererApiCommon;
use crate::core::graphic::hal::texture::TextureCommon;
use crate::core::graphic::hal::uniform_buffer::UniformBufferCommon;
use crate::core::graphic::hal::vertex_buffer::VertexBufferCommon;
use crate::core::graphic::shader::shader_program::ShaderProgramCommon;

pub type Backend = back::Backend;
pub type Instance = back::Instance;
pub type Surface = back::Surface;

pub type GraphicPipeline = GraphicPipelineCommon<back::Backend>;
pub type RendererApi<'a> = RendererApiCommon<'a, back::Backend>;
pub type Texture = TextureCommon<back::Backend>;
pub type UniformBuffer<T> = UniformBufferCommon<back::Backend, T>;
pub type VertexBuffer = VertexBufferCommon<back::Backend>;
pub type ShaderProgram = ShaderProgramCommon<Backend>;
