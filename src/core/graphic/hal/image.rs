use nalgebra_glm::{vec2, TVec2};
use std::io::Cursor;

pub struct Image {
    pub stride: usize,
    image: Vec<u8>,
    size: TVec2<u32>,
}

impl Image {
    pub fn new_with_format(binaries: &[u8], format: image::ImageFormat) -> Result<Image, ()> {
        let image = match image::load(Cursor::new(binaries), format) {
            Ok(x) => x,
            Err(_) => {
                return Err(());
            }
        }
        .to_rgba();

        let (width, height) = image.dimensions();
        Ok(Image {
            image: image.to_vec(),
            stride: 4usize,
            size: vec2(width, height),
        })
    }

    pub fn new(raw: Vec<u8>, size: TVec2<u32>) -> Image {
        let stride = 4usize;
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

    pub fn size(&self) -> &TVec2<u32> {
        &self.size
    }

    pub fn row(&self, y: usize) -> &[u8] {
        let width = self.size.y;
        &self.image[y * (width as usize) * self.stride..(y + 1) * (width as usize) * self.stride]
    }
}
