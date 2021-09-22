use std::borrow::Cow;
use wgpu::ShaderSource;

pub fn get_standard_2d_shader_source() -> ShaderSource<'static> {
    wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/standard_2d.wgsl")))
}
