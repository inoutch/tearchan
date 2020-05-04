use image::RgbaImage;
use nalgebra_glm::{vec2, TVec2};
use std::io::Cursor;

pub struct Image {
    pub stride: usize,
    image: RgbaImage,
    size: TVec2<u32>,
}

impl Image {
    pub fn new(binaries: &[u8], format: image::ImageFormat) -> Result<Image, ()> {
        let image = match image::load(Cursor::new(binaries), format) {
            Ok(x) => x,
            Err(_) => {
                return Err(());
            }
        }
        .to_rgba();
        let (width, height) = image.dimensions();
        Ok(Image {
            image,
            stride: 4usize,
            size: vec2(width, height),
        })
    }

    pub fn size(&self) -> &TVec2<u32> {
        &self.size
    }

    pub fn row(&self, y: usize) -> &[u8] {
        let width = self.size.y;
        &(*self.image)[y * (width as usize) * self.stride..(y + 1) * (width as usize) * self.stride]
    }
}
