use crate::math::rect::Rect3;
use nalgebra_glm::{vec3, TVec3};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::collections::{hash_map, HashMap};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Array3D<T> {
    data: HashMap<i32, HashMap<i32, HashMap<i32, T>>>,
    rect: Rect3<i32>,
}

impl<T> Array3D<T>
where
    T: Clone,
{
    pub fn new() -> Array3D<T> {
        Array3D {
            data: HashMap::new(),
            rect: Rect3::default(),
        }
    }

    pub fn get(&self, position: &TVec3<i32>) -> Option<&T> {
        if let Some(rows) = self.data.get(&position.z) {
            if let Some(cols) = rows.get(&position.y) {
                return cols.get(&position.x);
            }
        }
        None
    }

    pub fn get_mut(&mut self, position: &TVec3<i32>) -> Option<&mut T> {
        if let Some(rows) = self.data.get_mut(&position.z) {
            if let Some(cols) = rows.get_mut(&position.y) {
                return cols.get_mut(&position.x);
            }
        }
        None
    }

    pub fn get_or_put<F>(&mut self, position: &TVec3<i32>, putter: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if self.get(position).is_none() {
            self.set(position, putter());
            return self.get(position).unwrap();
        }
        self.get(position).unwrap()
    }

    pub fn get_mut_or_put<F>(&mut self, position: &TVec3<i32>, putter: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        if self.get(position).is_none() {
            self.set(position, putter());
            return self.get_mut(position).unwrap();
        }
        self.get_mut(position).unwrap()
    }

    pub fn set(&mut self, position: &TVec3<i32>, value: T) {
        let data = &mut self.data;
        let rows = get_mut_or_insert!(data, position.z, { HashMap::new() });
        let cols = get_mut_or_insert!(rows, position.y, { HashMap::new() });
        cols.insert(position.x, value);

        self.rect.origin.x = min(self.rect.origin.x, position.x);
        self.rect.origin.y = min(self.rect.origin.y, position.y);
        self.rect.origin.z = min(self.rect.origin.z, position.z);

        self.rect.size.x = max(self.rect.size.x, position.x + 1 - self.rect.origin.x);
        self.rect.size.y = max(self.rect.size.y, position.y + 1 - self.rect.origin.y);
        self.rect.size.z = max(self.rect.size.z, position.z + 1 - self.rect.origin.z);
    }

    pub fn rect(&self) -> &Rect3<i32> {
        &self.rect
    }

    pub fn add(&mut self, array: &Array3D<T>) {
        for z in array.data.iter() {
            for y in z.1.iter() {
                for x in y.1.iter() {
                    self.set(&vec3(*x.0, *y.0, *z.0), x.1.clone());
                }
            }
        }
    }

    pub fn remove(&mut self, position: &TVec3<i32>) -> Option<T> {
        if let Some(rows) = self.data.get_mut(&position.z) {
            if let Some(cols) = rows.get_mut(&position.y) {
                return cols.remove(&position.x);
            }
        }
        None
    }

    #[inline]
    pub fn iter(&self) -> Iter<T> {
        let mut iter_z = self.data.iter();
        let (z, iter_y) = match iter_z.next() {
            Some((z, map)) => {
                let mut iter = map.iter();
                iter.next();
                (*z, Some(iter))
            }
            None => (0, None),
        };
        let (y, iter_x) = match match self.data.get(&z) {
            Some(value) => value.iter().next(),
            None => None,
        } {
            Some((y, map)) => (*y, Some(map.iter())),
            None => (0, None),
        };
        Iter {
            iter_z,
            iter_y,
            iter_x,
            index: vec3(0, y, z),
        }
    }
}

pub struct Iter<'a, T: 'a> {
    iter_z: hash_map::Iter<'a, i32, HashMap<i32, HashMap<i32, T>>>,
    iter_y: Option<hash_map::Iter<'a, i32, HashMap<i32, T>>>,
    iter_x: Option<hash_map::Iter<'a, i32, T>>,
    index: TVec3<i32>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (TVec3<i32>, &'a T);

    #[inline]
    fn next(&mut self) -> Option<(TVec3<i32>, &'a T)> {
        loop {
            match match &mut self.iter_x {
                None => None,
                Some(iter) => iter.next(),
            } {
                Some((x, value)) => {
                    self.index.x = *x;
                    return Some((self.index.clone_owned(), value));
                }
                None => match match &mut self.iter_y {
                    None => None,
                    Some(iter) => iter.next(),
                } {
                    Some((y, map)) => {
                        self.index.y = *y;
                        self.iter_x = Some(map.iter());
                    }
                    None => match self.iter_z.next() {
                        Some((z, map)) => {
                            self.index.z = *z;
                            self.iter_y = Some(map.iter());
                        }
                        None => return None,
                    },
                },
            }
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl<'a, T> Iter<'a, T> {
    pub fn len(mut self) -> usize {
        let mut i = 0;
        while self.next().is_some() {
            i += 1;
        }
        i
    }
}

#[cfg(test)]
mod test {
    use crate::array_3d::Array3D;
    use nalgebra_glm::vec3;

    #[test]
    fn test_set() {
        let mut array: Array3D<i32> = Array3D::default();
        array.set(&vec3(0, 0, 0), 11);
        {
            let rect = array.rect();
            assert_eq!(&rect.origin, &vec3(0, 0, 0));
            assert_eq!(&rect.size, &vec3(1, 1, 1));
            assert_eq!(array.get(&vec3(0, 0, 0)), Some(&11));
        }

        array.set(&vec3(4, -4, 7), -245);
        {
            let rect = array.rect();
            assert_eq!(&rect.origin, &vec3(0, -4, 0));
            assert_eq!(&rect.size, &vec3(5, 1, 8));
            assert_eq!(array.get(&vec3(4, -4, 7)), Some(&-245));
        }
    }

    #[test]
    fn test_serialization() {
        let mut array1: Array3D<i32> = Array3D::default();
        array1.set(&vec3(0, 0, 0), 11);
        array1.set(&vec3(4, -4, 7), -245);

        let json = serde_json::to_string(&array1).unwrap();
        let array2: Array3D<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(array2.get(&vec3(0, 0, 0)), Some(&11));
        assert_eq!(array2.get(&vec3(4, -4, 7)), Some(&-245));
    }

    #[test]
    fn test_iterator() {
        let mut array1: Array3D<i32> = Array3D::default();
        array1.set(&vec3(1, 0, 0), 11);
        array1.set(&vec3(4, -4, 7), -245);

        let mut array = array1
            .iter()
            .map(|(index, value)| vec![index.x, index.y, index.z, *value])
            .collect::<Vec<_>>();
        array.sort_by(|a, b| a[0].cmp(&b[0]));
        assert_eq!(array, vec![vec![1, 0, 0, 11], vec![4, -4, 7, -245]]);
    }

    #[test]
    fn test_iterator_empty() {
        let array1: Array3D<i32> = Array3D::default();
        let array = array1
            .iter()
            .map(|(index, value)| vec![index.x, index.y, index.z, *value]);
        assert_eq!(array.count(), 0);
    }

    #[test]
    fn test_get_or_put() {
        let mut array: Array3D<i32> = Array3D::default();
        array.set(&vec3(0, 0, 0), 12);
        assert_eq!(array.get_or_put(&vec3(1, 1, 1), || 5), &5);
        assert_eq!(array.get_or_put(&vec3(0, 0, 0), || 6), &12);
    }

    #[test]
    fn test_get_mut_or_put() {
        let mut array: Array3D<i32> = Array3D::default();
        array.set(&vec3(0, 0, 0), 12);
        assert_eq!(array.get_mut_or_put(&vec3(1, 1, 1), || 5), &5);
        assert_eq!(array.get_mut_or_put(&vec3(0, 0, 0), || 6), &12);
    }
}
