use crate::hal::render_bundle::RenderBundleCommon;
use crate::hal::texture::{TextureCommon, TextureConfig};
use crate::image::Image;
use gfx_hal::Backend;
use nalgebra_glm::{vec2, vec4, TVec2, Vec2};
use rusttype::{point, Font, GlyphId, Scale};
use std::collections::HashMap;
use tearchan_utility::rect::{rect2, Rect2};

const DEFAULT_TEXTURE_SIZE_WIDTH: u32 = 512;
const DEFAULT_TEXTURE_SIZE_HEIGHT: u32 = 512;
const IMAGE_STRIDE: usize = 4usize; // r, g, b, a

#[derive(Debug)]
pub enum FontTextureError {
    FailedToLoad,
}

pub struct FontTextureCommon<B: Backend> {
    render_bundle: RenderBundleCommon<B>,
    texture: TextureCommon<B>,
    image_size: TVec2<u32>,
    mapped_size: Vec2,
    font_data: Vec<u8>,
    database: HashMap<GlyphId, Rect2<f32>>,
    h_max: f32,
    scale: f32,
}

impl<B: Backend> FontTextureCommon<B> {
    pub fn new(
        render_bundle: &RenderBundleCommon<B>,
        font_data: Vec<u8>,
        texture_config: TextureConfig,
        scale: f32,
    ) -> Result<Self, FontTextureError> {
        let _ = Font::try_from_bytes(&font_data).ok_or(FontTextureError::FailedToLoad)?;
        let image_size = vec2(DEFAULT_TEXTURE_SIZE_WIDTH, DEFAULT_TEXTURE_SIZE_HEIGHT);
        let database = HashMap::new();
        let mapped_size = vec2(0.0f32, 0.0f32);
        let h_max = 0.0f32;
        let image = create_empty_image(&image_size);
        let texture = TextureCommon::new(&render_bundle, &image, texture_config);

        Ok(FontTextureCommon {
            render_bundle: render_bundle.clone(),
            texture,
            image_size,
            mapped_size,
            font_data,
            database,
            h_max,
            scale,
        })
    }

    pub fn register_characters(&mut self, characters: &str) {
        self.update_texture(characters).unwrap();
    }

    pub fn register_all_alphabets(&mut self) {
        let mut characters = String::new();
        for c in 'A'..='z' {
            characters.push(c);
        }
        self.update_texture(&characters).unwrap();
    }

    pub fn register_all_numbers(&mut self) {
        let mut characters = String::new();
        for c in '0'..='9' {
            characters.push(c);
        }
        self.update_texture(&characters).unwrap();
    }

    pub fn texture(&self) -> &TextureCommon<B> {
        &self.texture
    }

    fn update_texture(&mut self, characters: &str) -> Result<(), FontTextureError> {
        let font = Font::try_from_bytes(&self.font_data).ok_or(FontTextureError::FailedToLoad)?;
        let padding = vec2(2.0f32, 2.0f32);
        let mut retry;

        loop {
            retry = false;
            for glyph in font.layout(
                characters,
                Scale::uniform(self.scale),
                point(0.0f32, 0.0f32),
            ) {
                let id = glyph.id();
                if self.database.contains_key(&id) {
                    // Already registered
                    continue;
                }

                let bounding_box = match glyph.pixel_bounding_box() {
                    Some(bounding_box) => bounding_box,
                    None => continue,
                };

                let width = bounding_box.width() as f32 + padding.x;
                let height = bounding_box.height() as f32 + padding.y;

                if self.mapped_size.x + width >= self.image_size.x as f32 {
                    // New line
                    self.mapped_size.x = 0.0f32;
                    self.mapped_size.y += self.h_max;
                    self.h_max = 0.0f32;
                }

                let character_rect = rect2(
                    self.mapped_size.x,
                    self.mapped_size.y,
                    bounding_box.width() as f32,
                    bounding_box.height() as f32,
                );
                self.mapped_size.x += width; // Next character place
                self.h_max = self.h_max.max(height);

                if self.mapped_size.y as f32 + self.h_max >= self.image_size.y as f32 {
                    self.image_size = vec2(self.image_size.x * 2, self.image_size.y * 2);
                    let image = create_empty_image(&self.image_size);
                    self.texture = TextureCommon::new(
                        &self.render_bundle,
                        &image,
                        self.texture.config().clone(),
                    );
                    self.mapped_size = vec2(0.0f32, 0.0f32);
                    self.database.clear();

                    retry = true;
                    break;
                }
                let offset = vec2(
                    character_rect.origin.x as u32,
                    character_rect.origin.y as u32,
                );
                let size = vec2(character_rect.size.x as u32, character_rect.size.y as u32);
                let mut image = create_empty_image(&size);

                self.database.insert(id, character_rect);
                glyph.draw(|x, y, alpha| {
                    image.set_color(
                        &vec2(x as usize, y as usize),
                        &vec4(255, 255, 255, (255.0f32 * alpha) as u8),
                    );
                });

                self.texture
                    .image_resource_mut()
                    .copy(&image, &offset)
                    .unwrap();
            }

            if !retry {
                break;
            }
        }
        Ok(())
    }
}

fn create_empty_image(image_size: &TVec2<u32>) -> Image {
    Image::new(
        vec![0u8; image_size.x as usize * image_size.y as usize * IMAGE_STRIDE],
        image_size.clone_owned(),
    )
}
