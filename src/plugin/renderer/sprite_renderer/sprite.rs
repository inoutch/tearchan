use nalgebra_glm::{vec2, Vec2};
use std::collections::HashMap;
use tearchan_utility::texture::{TextureAtlas, TextureFrame};

pub struct Sprite {
    texture_atlas: TextureAtlas,
    texture_key_map: HashMap<String, usize>,
    current_key: String,
    size: Vec2,
}

impl Sprite {
    pub fn new(texture_atlas: TextureAtlas) -> Self {
        let frame = texture_atlas
            .frames
            .first()
            .expect("There must be at least one or more frames");
        let size = vec2(frame.source.w as _, frame.source.h as _);
        let texture_key_map: HashMap<_, _> = texture_atlas
            .frames
            .iter()
            .enumerate()
            .map(|(i, frame)| (frame.key.to_string(), i as usize))
            .collect();
        let current_key = frame.key.clone();
        Sprite {
            texture_atlas,
            texture_key_map,
            current_key,
            size,
        }
    }

    pub fn texture_atlas(&self) -> &TextureAtlas {
        &self.texture_atlas
    }

    pub fn current_frame(&self) -> Option<&TextureFrame> {
        self.texture_key_map
            .get(&self.current_key)
            .and_then(|frame| self.texture_atlas.frames.get(*frame))
    }

    pub fn set_atlas(&mut self, key: String) {
        if let Some(frame_index) = self.texture_key_map.get(&key) {
            let frame = &self.texture_atlas.frames[*frame_index];
            self.current_key = key;
            self.size = vec2(frame.source.w as _, frame.source.h as _);
        }
    }
}
