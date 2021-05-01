use image::ImageResult;
use nalgebra_glm::{vec2, vec4, TVec2, TVec4};
use std::io::Cursor;

pub struct Image {
    stride: usize,
    image: Vec<u8>,
    size: TVec2<u32>,
}

impl Image {
    pub fn new_with_format(binaries: &[u8], format: image::ImageFormat) -> ImageResult<Image> {
        let image = match image::load(Cursor::new(binaries), format) {
            Ok(x) => x,
            Err(e) => {
                return Err(e);
            }
        }
        .to_rgba8();

        let (width, height) = image.dimensions();
        Ok(Image {
            image: image.to_vec(),
            stride: 4usize,
            size: vec2(width, height),
        })
    }

    pub fn new(raw: Vec<u8>, size: TVec2<u32>) -> Image {
        Self::new_with_stride(raw, size, 4usize)
    }

    pub fn new_with_stride(raw: Vec<u8>, size: TVec2<u32>, stride: usize) -> Image {
        debug_assert!(
            raw.len() == (size.x * size.y) as usize * stride,
            "invalid image size"
        );
        Image {
            image: raw,
            stride,
            size,
        }
    }

    pub fn new_empty() -> Image {
        Image::new(vec![255u8, 255u8, 255u8, 255u8], vec2(1, 1))
    }

    pub fn new_empty_with_size(size: TVec2<u32>) -> Self {
        let pixels = vec![0; (size.x * size.y) as usize * 4usize];
        Image::new(pixels, size)
    }

    pub fn new_empty_with_size_with_stride(size: TVec2<u32>, stride: usize) -> Self {
        let pixels = vec![0; (size.x * size.y) as usize * stride];
        Image::new_with_stride(pixels, size, stride)
    }

    pub fn size(&self) -> &TVec2<u32> {
        &self.size
    }

    pub fn row(&self, y: usize) -> &[u8] {
        let width = self.size.x;
        &self.image[y * (width as usize) * self.stride..(y + 1) * (width as usize) * self.stride]
    }

    pub fn set_color(&mut self, point: &TVec2<usize>, color: &TVec4<u8>) {
        let width = self.size.x;
        let index = (point.y * (width as usize) + point.x) * self.stride;
        self.image[index] = color.x;
        self.image[index + 1] = color.y;
        self.image[index + 2] = color.z;
        self.image[index + 3] = color.w;
    }

    pub fn get_color(&self, point: &TVec2<usize>) -> TVec4<u8> {
        let width = self.size.x;
        let index = (point.y * (width as usize) + point.x) * self.stride;
        vec4(
            self.image[index],
            self.image[index + 1],
            self.image[index + 2],
            self.image[index + 3],
        )
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    pub fn bytes(&self) -> &[u8] {
        &self.image
    }
}
