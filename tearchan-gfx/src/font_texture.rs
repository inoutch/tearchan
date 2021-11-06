use crate::image::Image;
use crate::texture::Texture;
use nalgebra_glm::{vec2, vec4, TVec2, Vec2};
use rusttype::{point, Font, GlyphId, IntoGlyphId, Scale};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use tearchan_util::math::rect::{rect2, Rect2};
use tearchan_util::math::vec::vec4_white;
use tearchan_util::mesh::square::{
    create_square_colors, create_square_indices_with_offset, create_square_positions,
    create_square_texcoords, create_square_texcoords_inv,
};
use tearchan_util::mesh::{IndexType, Mesh, MeshBuilder};
use wgpu::{Extent3d, ImageDataLayout, TextureAspect};

const DEFAULT_TEXTURE_SIZE_WIDTH: u32 = 512;
const DEFAULT_TEXTURE_SIZE_HEIGHT: u32 = 512;
const IMAGE_STRIDE: usize = 4usize; // r, g, b, a

#[derive(Debug)]
pub enum FontTextureError {
    FailedToLoad,
}

impl Display for FontTextureError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for FontTextureError {}

pub struct FontTexture {
    texture: Texture,
    texture_label: String,
    font_data: Vec<u8>,
    actual_size: TVec2<u32>,
    virtual_size: Vec2,
    dictionary: HashMap<GlyphId, Rect2<f32>>,
    h_max: f32,
    scale: f32,
}

impl Deref for FontTexture {
    type Target = Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl FontTexture {
    pub fn new(
        device: &wgpu::Device,
        font_data: Vec<u8>,
        scale: f32,
        sampler: wgpu::Sampler,
        label: &str,
    ) -> Result<Self, FontTextureError> {
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let _ = Font::try_from_bytes(&font_data).ok_or(FontTextureError::FailedToLoad)?;
        let actual_size = vec2(DEFAULT_TEXTURE_SIZE_WIDTH, DEFAULT_TEXTURE_SIZE_HEIGHT);
        let dictionary = HashMap::new();
        let virtual_size = vec2(0.0f32, 0.0f32);
        let h_max = 0.0f32;
        let (texture, texture_view) =
            create_texture_bundle(device, actual_size.x, actual_size.y, format, label);

        Ok(FontTexture {
            texture: Texture::new(texture, texture_view, sampler, format),
            texture_label: label.to_string(),
            actual_size,
            virtual_size,
            font_data,
            dictionary,
            h_max,
            scale,
        })
    }

    pub fn write_characters(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        characters: &str,
    ) -> Result<(), FontTextureError> {
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
                if self.dictionary.contains_key(&id) {
                    // Already registered
                    continue;
                }

                let bounding_box = match glyph.pixel_bounding_box() {
                    Some(bounding_box) => bounding_box,
                    None => continue,
                };

                let width = bounding_box.width() as f32 + padding.x;
                let height = bounding_box.height() as f32 + padding.y;

                if self.virtual_size.x + width >= self.actual_size.x as f32 {
                    // New line
                    self.virtual_size.x = 0.0f32;
                    self.virtual_size.y += self.h_max;
                    self.h_max = 0.0f32;
                }

                let character_rect = rect2(
                    self.virtual_size.x,
                    self.virtual_size.y,
                    bounding_box.width() as f32,
                    bounding_box.height() as f32,
                );
                self.virtual_size.x += width; // Next character place
                self.h_max = self.h_max.max(height);

                if self.virtual_size.y as f32 + self.h_max >= self.actual_size.y as f32 {
                    // Create new texture
                    self.actual_size = vec2(self.actual_size.x * 2, self.actual_size.y * 2);
                    let (texture, view) = create_texture_bundle(
                        device,
                        self.actual_size.x,
                        self.actual_size.y,
                        self.texture.format(),
                        &self.texture_label,
                    );
                    self.texture.texture = texture;
                    self.texture.view = view;
                    self.virtual_size = vec2(0.0f32, 0.0f32);
                    self.dictionary.clear();

                    retry = true;
                    break;
                }

                let character_offset = vec2(
                    character_rect.origin.x as u32,
                    character_rect.origin.y as u32,
                );
                let character_size =
                    vec2(character_rect.size.x as u32, character_rect.size.y as u32);
                let mut character_image = create_empty_image(&character_size);
                self.dictionary.insert(id, character_rect);
                glyph.draw(|x, y, alpha| {
                    character_image.set_color(
                        &vec2(x as usize, y as usize),
                        &vec4(255, 255, 255, (255.0f32 * alpha) as u8),
                    );
                });

                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: self.texture.texture(),
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: character_offset.x,
                            y: character_offset.y,
                            z: 0,
                        },
                        aspect: TextureAspect::All,
                    },
                    character_image.bytes(),
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            std::num::NonZeroU32::new(character_image.size().x * 4).unwrap(),
                        ),
                        rows_per_image: None,
                    },
                    Extent3d {
                        width: character_size.x,
                        height: character_size.y,
                        depth_or_array_layers: 1,
                    },
                );
            }

            if !retry {
                break;
            }
        }
        Ok(())
    }

    pub fn create_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        characters: &str,
    ) -> Result<(Mesh, Vec2), FontTextureError> {
        self.write_characters(device, queue, characters)?;

        let mut size = vec2(0.0f32, 0.0f32);
        let font = rusttype::Font::try_from_bytes(&self.font_data).unwrap();
        let scale = Scale::uniform(self.scale);
        let v_metrics = font.v_metrics(scale);
        let line_height = v_metrics.ascent;

        let mut indices = vec![];
        let mut positions = vec![];
        let mut colors = vec![];
        let mut texcoords = vec![];
        let texture_size = vec2(self.actual_size.x as f32, self.actual_size.y as f32);
        let prev_glyph_id: Option<GlyphId> = None;

        let mut x = 0.0f32;
        let mut y = -line_height - v_metrics.descent;
        for glyph in font.layout(characters, scale, point(0.0f32, 0.0f32)) {
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
            let texture_rect = &self.dictionary[&glyph.id()];
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
            colors.append(&mut create_square_colors(vec4_white()));
            texcoords.append(&mut create_square_texcoords(&uv_rect));

            x += h_metrics.advance_width;
            size.x = size.x.max(x);
        }
        size.y = -y;

        for position in &mut positions {
            position.y += size.y;
        }
        Ok((
            MeshBuilder::new()
                .indices(indices)
                .positions(positions)
                .colors(colors)
                .texcoords(texcoords)
                .normals(vec![])
                .build()
                .unwrap(),
            size,
        ))
    }

    pub fn create_mesh_inv(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        characters: &str,
    ) -> Result<(Mesh, Vec2), FontTextureError> {
        self.write_characters(device, queue, characters)?;

        let mut size = vec2(0.0f32, 0.0f32);
        let font = rusttype::Font::try_from_bytes(&self.font_data).unwrap();
        let scale = Scale::uniform(self.scale);
        let v_metrics = font.v_metrics(scale);
        let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        let mut indices = vec![];
        let mut positions = vec![];
        let mut colors = vec![];
        let mut texcoords = vec![];
        let texture_size = vec2(self.actual_size.x as f32, self.actual_size.y as f32);

        let mut base_line = v_metrics.ascent;
        let mut prev_line_width = 0.0f32;
        let mut width = 0.0f32;

        for glyph in font.layout(characters, scale, point(0.0f32, 0.0f32)) {
            // new line
            let h_metrics = glyph.unpositioned().h_metrics();
            if glyph.id() == '\n'.into_glyph_id(&font) {
                width += h_metrics.advance_width;
                size.y += line_height;
                base_line += line_height;
                prev_line_width = width;
                size.x = size.x.max(width);
                continue;
            }
            let bounds = match glyph.pixel_bounding_box() {
                Some(bounding_box) => bounding_box,
                None => {
                    width += h_metrics.advance_width;
                    continue;
                }
            };

            let texture_rect = &self.dictionary[&glyph.id()];
            let uv_rect = rect2(
                texture_rect.origin.x / texture_size.x,
                texture_rect.origin.y / texture_size.y,
                texture_rect.size.x / texture_size.x,
                texture_rect.size.y / texture_size.y,
            );
            let rect = rect2(
                bounds.min.x as f32 - prev_line_width,
                base_line + bounds.min.y as f32,
                (bounds.max.x - bounds.min.x) as f32,
                (bounds.max.y - bounds.min.y) as f32,
            );
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions(&rect));
            colors.append(&mut create_square_colors(vec4_white()));
            texcoords.append(&mut create_square_texcoords_inv(&uv_rect));
            width = rect.origin.x + rect.size.x;
        }
        size.y += line_height;
        size.x = size.x.max(width);

        Ok((
            MeshBuilder::new()
                .indices(indices)
                .positions(positions)
                .colors(colors)
                .texcoords(texcoords)
                .normals(vec![])
                .build()
                .unwrap(),
            size,
        ))
    }
}

fn create_empty_image(image_size: &TVec2<u32>) -> Image {
    Image::new(
        vec![0u8; image_size.x as usize * image_size.y as usize * IMAGE_STRIDE],
        image_size.clone_owned(),
    )
}

fn create_texture_bundle(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture_desc = wgpu::TextureDescriptor {
        label: Some(label),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    };
    let texture = device.create_texture(&texture_desc);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
