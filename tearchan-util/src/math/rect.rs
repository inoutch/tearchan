use crate::compare::pick;
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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

pub fn intersect_rect3_i32(rect: &Rect3<i32>, p: &TVec3<i32>) -> bool {
    rect.origin.x <= p.x
        && p.x < rect.origin.x + rect.size.x
        && rect.origin.y <= p.y
        && p.y < rect.origin.y + rect.size.y
        && rect.origin.z <= p.z
        && p.z < rect.origin.z + rect.size.z
}

pub fn intersect_rect2_i32(rect: &Rect2<i32>, p: &TVec2<i32>) -> bool {
    rect.origin.x <= p.x
        && p.x < rect.origin.x + rect.size.x
        && rect.origin.y <= p.y
        && p.y < rect.origin.y + rect.size.y
}

pub fn pick_rect3_i32(rect: &Rect3<i32>, p: &TVec3<i32>) -> TVec3<i32> {
    vec3(
        pick(p.x, rect.origin.x, rect.origin.x + rect.size.x),
        pick(p.y, rect.origin.y, rect.origin.x + rect.size.y),
        pick(p.z, rect.origin.z, rect.origin.x + rect.size.z),
    )
}

#[cfg(test)]
mod test {
    use crate::math::rect::{intersect_rect2_i32, intersect_rect3_i32, Rect2, Rect3};
    use nalgebra_glm::{vec2, vec3};

    #[test]
    fn test_intersect_rect2_i32() {
        assert!(intersect_rect2_i32(
            &Rect2 {
                origin: vec2(-1, 2),
                size: vec2(1, 1)
            },
            &vec2(-1, 2)
        ));

        assert!(intersect_rect2_i32(
            &Rect2 {
                origin: vec2(-1, 2),
                size: vec2(2, 2)
            },
            &vec2(0, 3)
        ));

        assert!(!intersect_rect2_i32(
            &Rect2 {
                origin: vec2(-1, 2),
                size: vec2(2, 2)
            },
            &vec2(1, 3)
        ));
    }

    #[test]
    fn test_intersect_rect3_i32() {
        assert!(intersect_rect3_i32(
            &Rect3 {
                origin: vec3(-1, 2, 5),
                size: vec3(1, 1, 1)
            },
            &vec3(-1, 2, 5)
        ));

        assert!(intersect_rect3_i32(
            &Rect3 {
                origin: vec3(-1, 2, 5),
                size: vec3(2, 2, 2)
            },
            &vec3(0, 3, 6)
        ));

        assert!(!intersect_rect3_i32(
            &Rect3 {
                origin: vec3(-1, 2, 5),
                size: vec3(2, 2, 2)
            },
            &vec3(1, 3, 7)
        ));
    }
}
