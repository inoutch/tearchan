use crate::core::graphic::hal::backend::{Graphics, Texture};
use crate::core::graphic::hal::texture::TextureConfig;
use crate::core::graphic::image::Image;
use crate::math::mesh::square::{
    create_square_colors, create_square_indices_with_offset, create_square_positions,
    create_square_texcoords,
};
use crate::math::mesh::{IndexType, Mesh, MeshBuilder};
use crate::math::rect::{rect2, Rect2};
use crate::math::vec::make_vec4_white;
use nalgebra_glm::{vec2, vec4, TVec2, Vec2};
use rusttype::{point, GlyphId, IntoGlyphId, Scale};
use std::collections::HashMap;

const DEFAULT_TEXTURE_SIZE_WIDTH: u32 = 512;
const DEFAULT_TEXTURE_SIZE_HEIGHT: u32 = 512;
const IMAGE_STRIDE: usize = 4usize;

#[derive(Debug)]
pub enum FontTextureError {
    FailedToLoad,
}

pub struct FontTexture {
    texture: Texture,
    image: Image,
    image_size: TVec2<u32>,
    data: Vec<u8>,
    map: HashMap<GlyphId, Rect2<f32>>,
    text: String,
    text_is_changed: bool,
    current_size: Vec2,
    h_max: f32,
    scale: f32,
}

impl FontTexture {
    pub fn new(
        graphics: &mut Graphics,
        data: Vec<u8>,
        text: &str,
        scale: f32,
    ) -> Result<Self, FontTextureError> {
        let mut image_size =
            vec2(DEFAULT_TEXTURE_SIZE_WIDTH, DEFAULT_TEXTURE_SIZE_HEIGHT).clone_owned();
        let mut map: HashMap<GlyphId, Rect2<f32>> = HashMap::new();
        let mut current_size = vec2(0.0f32, 0.0f32);
        let mut h_max: f32 = 0.0f32;
        let mut image = Image::new(
            vec![0u8; image_size.x as usize * image_size.y as usize * IMAGE_STRIDE],
            image_size.clone_owned(),
        );

        FontTexture::update_image(
            (&data, text, scale),
            &mut image,
            &mut map,
            &mut image_size,
            &mut current_size,
            &mut h_max,
        )?;

        let texture = graphics.create_texture(&image, TextureConfig::default());
        Ok(FontTexture {
            texture,
            image,
            image_size,
            data,
            map,
            text: text.to_string(),
            text_is_changed: false,
            current_size,
            h_max,
            scale,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        if self.text != text {
            self.text = text;
            FontTexture::update_image(
                (&self.data, &self.text, 100.0f32),
                &mut self.image,
                &mut self.map,
                &mut self.image_size,
                &mut self.current_size,
                &mut self.h_max,
            )
            .unwrap();

            self.text_is_changed = true;
        }
    }

    pub fn flush(&mut self, graphics: &mut Graphics) {
        if self.text_is_changed {
            self.texture = graphics.create_texture(&self.image, TextureConfig::default());
            self.text_is_changed = false;
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn build_mesh(&self) -> (Mesh, Vec2) {
        let mut size = vec2(0.0f32, 0.0f32);
        let font = rusttype::Font::try_from_bytes(&self.data)
            .ok_or_else(|| FontTextureError::FailedToLoad)
            .unwrap();
        let scale = Scale::uniform(self.scale);
        let v_metrics = font.v_metrics(scale);
        let line_height = v_metrics.ascent;

        let mut indices = vec![];
        let mut positions = vec![];
        let mut colors = vec![];
        let mut texcoords = vec![];
        let texture_size = vec2(self.texture.size().x as f32, self.texture.size().y as f32);
        let prev_glyph_id: Option<GlyphId> = None;

        let mut x = 0.0f32;
        let mut y = -line_height - v_metrics.descent;
        for glyph in font.layout(&self.text, scale, point(0.0f32, 0.0f32)) {
            if glyph.id() == '\n'.into_glyph_id(&font) {
                x = 0.0f32;
                y -= line_height;
                continue;
            }
            x -= match prev_glyph_id {
                None => 0.0f32,
                Some(id) => font.pair_kerning(scale, id, glyph.id()),
            };
            let h_metrics = glyph.unpositioned().h_metrics();
            let max_y = match glyph.pixel_bounding_box() {
                Some(bounding_box) => (bounding_box.max.y) as f32,
                None => {
                    x += h_metrics.advance_width;
                    continue;
                }
            };
            let texture_rect = &self.map[&glyph.id()];
            let uv_rect = rect2(
                texture_rect.origin.x / texture_size.x,
                texture_rect.origin.y / texture_size.y,
                texture_rect.size.x / texture_size.x,
                texture_rect.size.y / texture_size.y,
            );
            let rect = rect2(
                x + h_metrics.left_side_bearing,
                y - max_y,
                h_metrics.advance_width - h_metrics.left_side_bearing,
                texture_rect.size.y,
            );
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions(&rect));
            colors.append(&mut create_square_colors(make_vec4_white()));
            texcoords.append(&mut create_square_texcoords(&uv_rect));

            x += h_metrics.advance_width;
            size.x = size.x.max(x);
        }
        size.y = -y;

        for position in &mut positions {
            position.y += size.y;
        }
        (
            MeshBuilder::new()
                .indices(indices)
                .positions(positions)
                .colors(colors)
                .texcoords(texcoords)
                .normals(vec![])
                .build()
                .unwrap(),
            size,
        )
    }

    fn update_image(
        constants: (&[u8], &str, f32),
        image: &mut Image,
        map: &mut HashMap<GlyphId, Rect2<f32>>,
        image_size: &mut TVec2<u32>,
        current_size: &mut TVec2<f32>,
        h_max: &mut f32,
    ) -> Result<(), FontTextureError> {
        let font = rusttype::Font::try_from_bytes(constants.0)
            .ok_or_else(|| FontTextureError::FailedToLoad)?;
        let padding = vec2(2.0f32, 2.0f32);
        let mut retry;

        loop {
            retry = false;
            for glyph in font.layout(
                constants.1,
                Scale::uniform(constants.2),
                point(0.0f32, 0.0f32),
            ) {
                let id = glyph.id();
                if map.contains_key(&id) {
                    continue;
                }

                let bounding_box = match glyph.pixel_bounding_box() {
                    Some(bounding_box) => bounding_box,
                    None => continue,
                };
                let width = bounding_box.width() as f32 + padding.x;
                let height = bounding_box.height() as f32 + padding.y;

                if current_size.x + width >= image.size().x as f32 {
                    current_size.x = 0.0f32;
                    current_size.y += *h_max;
                    *h_max = 0.0f32;
                }

                let rect = rect2(
                    current_size.x,
                    current_size.y,
                    bounding_box.width() as f32,
                    bounding_box.height() as f32,
                );
                current_size.x += width;
                *h_max = h_max.max(height);

                if current_size.y + *h_max >= image.size().y as f32 {
                    *image_size = vec2(image_size.x * 2, image_size.y * 2);
                    *image = Image::new(
                        vec![0u8; image_size.x as usize * image_size.y as usize * IMAGE_STRIDE],
                        image_size.clone_owned(),
                    );
                    *current_size = vec2(0.0f32, 0.0f32);
                    map.clear();

                    retry = true;
                    break;
                }

                let ox = rect.origin.x as usize;
                let oy = rect.origin.y as usize;

                map.insert(id, rect);
                glyph.draw(|x, y, alpha| {
                    image.set_color(
                        &vec2(x as usize + ox as usize, y as usize + oy),
                        &vec4(255, 255, 255, (255.0f32 * alpha) as u8),
                    );
                });
            }

            if !retry {
                break;
            }
        }
        Ok(())
    }
}
