use nalgebra_glm::TVec2;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Array2D<T> {
    data: HashMap<i32, HashMap<i32, T>>,
    size_start: TVec2<i32>,
    size_end: TVec2<i32>,
}

impl<T> Array2D<T>
where
    T: Clone,
{
    pub fn get(&self, position: &TVec2<i32>) -> Option<&T> {
        if let Some(y) = self.data.get(&position.y) {
            return y.get(&position.x);
        }
        None
    }

    pub fn get_mut(&mut self, position: &TVec2<i32>) -> Option<&mut T> {
        if let Some(y) = self.data.get_mut(&position.y) {
            return y.get_mut(&position.x);
        }
        None
    }

    pub fn get_or_put<F>(&mut self, position: &TVec2<i32>, putter: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if self.get(position).is_none() {
            self.set(position, putter());
            return self.get(position).unwrap();
        }
        self.get(position).unwrap()
    }

    pub fn get_mut_or_put<F>(&mut self, position: &TVec2<i32>, putter: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        if self.get(position).is_none() {
            self.set(position, putter());
            return self.get_mut(position).unwrap();
        }
        self.get_mut(position).unwrap()
    }

    pub fn set(&mut self, position: &TVec2<i32>, value: T) {
        let data = &mut self.data;
        let cols = get_mut_or_insert!(data, position.y, { HashMap::new() });

        cols.insert(position.x, value);

        self.size_start.x = min(self.size_start.x, position.x);
        self.size_start.y = min(self.size_start.y, position.y);

        self.size_end.x = max(self.size_end.x, position.x + 1);
        self.size_end.y = max(self.size_end.y, position.y + 1);
    }

    pub fn remove(&mut self, position: TVec2<i32>) -> Option<T> {
        let data = &mut self.data;
        if let Some(y) = data.get_mut(&position.y) {
            return y.remove(&position.x);
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::array_2d::Array2D;
    use nalgebra_glm::vec2;

    #[test]
    fn test_set() {
        let mut array: Array2D<i32> = Array2D::default();
        array.set(&vec2(0, 0), 1);
        array.set(&vec2(1, 0), 3);
        array.set(&vec2(1, 2), 6);

        assert_eq!(array.get(&vec2(0, 0)), Some(&1));
        assert_eq!(array.get(&vec2(0, 1)), None);
        assert_eq!(array.get(&vec2(1, 0)), Some(&3));
        assert_eq!(array.get(&vec2(1, 2)), Some(&6));
    }

    #[test]
    fn test_get_or_put() {
        let mut array: Array2D<i32> = Array2D::default();
        array.set(&vec2(0, 0), 12);
        assert_eq!(array.get_or_put(&vec2(1, 1), || 5), &5);
        assert_eq!(array.get_or_put(&vec2(0, 0), || 6), &12);
    }

    #[test]
    fn test_get_mut_or_put() {
        let mut array: Array2D<i32> = Array2D::default();
        array.set(&vec2(0, 0), 12);
        assert_eq!(array.get_mut_or_put(&vec2(1, 1), || 5), &5);
        assert_eq!(array.get_mut_or_put(&vec2(0, 0), || 6), &12);
    }
}
