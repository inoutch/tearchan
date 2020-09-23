use nalgebra_glm::{vec2, Vec2};
use serde::{Deserialize, Serialize};
use texture_packer::Frame;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureFrame {
    pub key: String,
    pub rect: Rect,
    pub rotated: bool,
    pub trimmed: bool,
    pub source: Rect,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Size {
    w: u32,
    h: u32,
}

impl Size {
    pub fn new(w: u32, h: u32) -> Self {
        Size { w, h }
    }

    pub fn to_vec2(&self) -> Vec2 {
        vec2(self.w as f32, self.h as f32)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureAtlas {
    pub image: String,
    pub size: Size,
    pub frames: Vec<TextureFrame>,
}

impl From<texture_packer::Rect> for Rect {
    fn from(r: texture_packer::Rect) -> Self {
        Rect {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
        }
    }
}

impl From<texture_packer::Frame> for TextureFrame {
    fn from(f: Frame) -> Self {
        TextureFrame {
            key: f.key,
            rect: Rect::from(f.frame),
            rotated: f.rotated,
            trimmed: f.trimmed,
            source: Rect::from(f.source),
        }
    }
}
