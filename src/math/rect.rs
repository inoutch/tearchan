use nalgebra_glm::{vec2, vec3, TVec2, TVec3};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rect2<T>
where
    T: 'static + PartialEq + Clone + Copy + Debug,
{
    pub origin: TVec2<T>,
    pub size: TVec2<T>,
}

pub fn rect2<T>(ox: T, oy: T, sx: T, sy: T) -> Rect2<T>
where
    T: 'static + PartialEq + Clone + Copy + Debug,
{
    Rect2 {
        origin: vec2(ox, oy),
        size: vec2(sx, sy),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rect3<T>
where
    T: 'static + PartialEq + Clone + Copy + Debug,
{
    pub origin: TVec3<T>,
    pub size: TVec3<T>,
}

pub fn rect3<T>(ox: T, oy: T, oz: T, sx: T, sy: T, sz: T) -> Rect3<T>
where
    T: 'static + PartialEq + Clone + Copy + Debug,
{
    Rect3 {
        origin: vec3(ox, oy, oz),
        size: vec3(sx, sy, sz),
    }
}
