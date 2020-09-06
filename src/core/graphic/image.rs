use image::ImageResult;
use nalgebra_glm::{vec2, TVec2, TVec4};
use std::io::Cursor;

pub struct Image {
    pub stride: usize,
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
}

#[cfg(test)]
mod test {
    use crate::core::graphic::image::Image;
    use nalgebra_glm::{vec2, vec4};

    #[test]
    fn test_row() {
        let colors = [
            vec4(0u8, 255u8, 255u8, 255u8),
            vec4(255u8, 0u8, 255u8, 255u8),
            vec4(255u8, 255u8, 0u8, 255u8),
            vec4(255u8, 255u8, 255u8, 0u8),
            vec4(0u8, 0u8, 255u8, 255u8),
            vec4(255u8, 0u8, 0u8, 255u8),
        ];
        let bytes = colors
            .iter()
            .map(|x| x.data.to_vec())
            .flatten()
            .collect::<Vec<u8>>();

        assert_eq!(bytes.len(), 24);

        let image = Image::new(bytes, vec2(2, 3));
        assert_eq!(
            image.row(0),
            &[0u8, 255u8, 255u8, 255u8, 255u8, 0u8, 255u8, 255u8]
        );
        assert_eq!(
            image.row(1),
            &[255u8, 255u8, 0u8, 255u8, 255u8, 255u8, 255u8, 0u8]
        );
        assert_eq!(
            image.row(2),
            &[0u8, 0u8, 255u8, 255u8, 255u8, 0u8, 0u8, 255u8]
        );
    }
}
