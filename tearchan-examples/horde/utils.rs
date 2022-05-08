use nalgebra_glm::{vec2, TVec2, Vec2};
use std::ops::{Add, Div, Mul, Sub};
use tearchan::gfx::wgpu::{
    Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub const CELL_SCALE_SIZE: f32 = 0.1f32;

pub fn calc_position_from_ratio(start: &Vec2, end: &Vec2, ratio: f32) -> Vec2 {
    vec2(
        calc_value_from_ratio(start.x, end.x, ratio),
        calc_value_from_ratio(start.y, end.y, ratio),
    )
}

pub fn calc_value_from_ratio<T>(start: T, end: T, ratio: T) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Mul<Output = T>,
{
    start + (end - start) * ratio
}

pub fn calc_center_from_scaled_position(scaled_position: &TVec2<i32>) -> Vec2 {
    vec2(
        (scaled_position.x) as f32 * CELL_SCALE_SIZE + CELL_SCALE_SIZE / 2.0f32,
        (scaled_position.y) as f32 * CELL_SCALE_SIZE + CELL_SCALE_SIZE / 2.0f32,
    )
}

pub fn create_texture_view(device: &Device, queue: &Queue) -> TextureView {
    let size = 1u32;
    let texel = vec![255, 255, 255, 255];
    let texture_extent = Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING
            | TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::COPY_DST,
    });
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    queue.write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &texel,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(std::num::NonZeroU32::new(4 * size).unwrap()),
            rows_per_image: None,
        },
        texture_extent,
    );
    texture_view
}
