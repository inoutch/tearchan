use nalgebra_glm::{rotate, scale, translate, vec2, vec3, Mat4, Vec2, Vec3, Vec4};
use std::collections::HashMap;
use tearchan_utility::math::vec::{vec2_zero, vec3_one, vec3_zero, vec4_white};
use tearchan_utility::mesh::square::{
    create_square_positions_from_frame, create_square_texcoords_from_frame,
};
use tearchan_utility::mesh::{PositionArray, TexcoordArray};
use tearchan_utility::texture::{TextureAtlas, TextureFrame};

pub struct Sprite {
    texture_atlas: TextureAtlas,
    texture_key_map: HashMap<String, usize>,
    current_key: String,
    position: Vec3,
    anchor_point: Vec2,
    rotation_radian: f32,
    rotation_axis: Vec3,
    scale: Vec3,
    color: Vec4,
    size: Vec2,
    // Changes
    is_position_changed: bool,
    is_rotation_changed: bool,
    is_scale_changed: bool,
    is_color_changed: bool,
    is_frame_changed: bool,
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
            position: vec3_zero(),
            anchor_point: vec2_zero(),
            rotation_radian: 0.0f32,
            rotation_axis: vec3(0.0f32, 0.0f32, 1.0f32),
            scale: vec3_one(),
            color: vec4_white(),
            size,
            is_position_changed: false,
            is_rotation_changed: false,
            is_scale_changed: false,
            is_color_changed: false,
            is_frame_changed: false,
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

    pub fn position(&self) -> &Vec3 {
        &self.position
    }

    pub fn anchor_point(&self) -> &Vec2 {
        &self.anchor_point
    }

    pub fn rotation_radian(&self) -> f32 {
        self.rotation_radian
    }

    pub fn rotation_axis(&self) -> &Vec3 {
        &self.rotation_axis
    }

    pub fn scale(&self) -> &Vec3 {
        &self.scale
    }

    pub fn color(&self) -> &Vec4 {
        &self.color
    }

    pub fn is_position_changed(&self) -> bool {
        self.is_position_changed
    }

    pub fn is_rotation_changed(&self) -> bool {
        self.is_rotation_changed
    }

    pub fn is_scale_changed(&self) -> bool {
        self.is_scale_changed
    }

    pub fn is_color_changed(&self) -> bool {
        self.is_color_changed
    }

    pub fn is_frame_changed(&self) -> bool {
        self.is_frame_changed
    }

    pub fn transform_anchor_point(&self) -> Vec3 {
        vec3(
            -self.size.x * self.anchor_point.x,
            -self.size.y * self.anchor_point.y,
            0.0f32,
        )
    }

    pub fn transform(&self) -> Mat4 {
        translate(
            &scale(
                &rotate(
                    &translate(&Mat4::identity(), &self.position),
                    self.rotation_radian,
                    &self.rotation_axis,
                ),
                &self.scale,
            ),
            &self.transform_anchor_point(),
        )
    }

    pub fn update_transform<F>(&self, callback: F)
    where
        F: FnOnce(Mat4),
    {
        if self.is_position_changed || self.is_scale_changed || self.is_rotation_changed {
            callback(self.transform());
        }
    }

    pub fn update_color<F>(&self, callback: F)
    where
        F: FnOnce(&Vec4),
    {
        if self.is_color_changed {
            callback(&self.color);
        }
    }

    pub fn update_frame<F>(&self, callback: F)
    where
        F: FnOnce(PositionArray, TexcoordArray),
    {
        if !self.is_frame_changed {
            return;
        }
        let frame = self.current_frame().unwrap();
        let positions = create_square_positions_from_frame(&vec2_zero(), frame);
        let texcoords =
            create_square_texcoords_from_frame(self.texture_atlas.size.to_vec2(), frame);
        callback(positions, texcoords);
    }

    pub fn set_atlas(&mut self, key: String) {
        if let Some(frame_index) = self.texture_key_map.get(&key) {
            if self.current_key != key {
                let frame = &self.texture_atlas.frames[*frame_index];
                self.current_key = key;
                self.size = vec2(frame.source.w as _, frame.source.h as _);
                self.is_frame_changed = true;
            }
        }
    }

    pub fn set_position(&mut self, position: Vec3) {
        if position != self.position {
            self.position = position;
            self.is_position_changed = true;
        }
    }

    pub fn set_anchor_position(&mut self, anchor_position: Vec2) {
        if anchor_position != self.anchor_point {
            self.anchor_point = anchor_position;
            self.is_position_changed = true;
        }
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        if scale != self.scale {
            self.scale = scale;
            self.is_scale_changed = true;
        }
    }

    pub fn set_color(&mut self, color: Vec4) {
        if color != self.color {
            self.color = color;
            self.is_color_changed = true;
        }
    }

    pub fn reset_changes(&mut self) {
        self.is_position_changed = false;
        self.is_rotation_changed = false;
        self.is_scale_changed = false;
        self.is_color_changed = false;
        self.is_frame_changed = false;
    }
}
