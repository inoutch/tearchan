use image::ImageResult;
use std::io::Cursor;

pub struct Bitmap {
    stride: usize,
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl Bitmap {
    pub fn new_with_format(binaries: &[u8], format: image::ImageFormat) -> ImageResult<Bitmap> {
        let image = match image::load(Cursor::new(binaries), format) {
            Ok(x) => x,
            Err(e) => {
                return Err(e);
            }
        }
        .to_rgba8();

        let (width, height) = image.dimensions();
        Ok(Bitmap {
            pixels: image.to_vec(),
            stride: 4usize,
            width,
            height,
        })
    }

    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Bitmap {
        let stride = 4usize;
        debug_assert!(
            pixels.len() == (width * height) as usize * stride,
            "invalid image size"
        );
        Bitmap {
            pixels,
            stride,
            width,
            height,
        }
    }

    pub fn new_empty() -> Bitmap {
        Bitmap::new(vec![255u8, 255u8, 255u8, 255u8], 1, 1)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn row(&self, y: usize) -> &[u8] {
        &self.pixels
            [y * (self.width as usize) * self.stride..(y + 1) * (self.width as usize) * self.stride]
    }

    pub fn set_color(&mut self, x: u32, y: u32, color: &(u8, u8, u8, u8)) {
        let index = (y * self.width + x) as usize * self.stride;
        self.pixels[index] = color.0;
        self.pixels[index + 1] = color.1;
        self.pixels[index + 2] = color.2;
        self.pixels[index + 3] = color.3;
    }

    pub fn stride(&self) -> usize {
        self.stride
    }
}
