use std::cmp;

pub fn pick<T: Ord>(p: T, min: T, max: T) -> T {
    cmp::min(max, cmp::max(p, min))
}
