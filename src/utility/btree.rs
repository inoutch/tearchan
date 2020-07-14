use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::btree_map::{Iter, Range, RangeMut};
use std::collections::{BTreeMap, VecDeque};
use std::ops::RangeBounds;

macro_rules! get_or_put {
    ($btree:expr, $key:expr, $putter:block) => {
        match $btree.get_mut(&$key) {
            Some(v) => v,
            None => {
                $btree.insert($key, $putter);
                $btree.get_mut(&$key).unwrap()
            }
        };
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicatableBTreeMap<K, V>
where
    K: Ord,
{
    btree: BTreeMap<K, VecDeque<V>>,
}

impl<K, V> DuplicatableBTreeMap<K, V>
where
    K: Ord,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self
    where
        K: Ord,
    {
        DuplicatableBTreeMap {
            btree: BTreeMap::new(),
        }
    }

    pub fn push_back(&mut self, key: K, value: V)
    where
        K: Ord + Copy,
    {
        get_or_put!(self.btree, key, { VecDeque::new() }).push_back(value);
    }

    pub fn push_front(&mut self, key: K, value: V)
    where
        K: Ord + Copy,
    {
        get_or_put!(self.btree, key, { VecDeque::new() }).push_front(value);
    }

    pub fn range<T: ?Sized, R>(&self, range: R) -> Range<'_, K, VecDeque<V>>
    where
        T: Ord,
        K: Borrow<T>,
        R: RangeBounds<T>,
    {
        self.btree.range(range)
    }

    pub fn range_mut<T: ?Sized, R>(&mut self, range: R) -> RangeMut<'_, K, VecDeque<V>>
    where
        T: Ord,
        K: Borrow<T>,
        R: RangeBounds<T>,
    {
        self.btree.range_mut(range)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.btree.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.btree.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, VecDeque<V>> {
        self.btree.iter()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.btree.clear();
    }
}

#[cfg(test)]
mod test {
    use crate::utility::btree::DuplicatableBTreeMap;

    #[test]
    pub fn test_btree() {
        let mut btree: DuplicatableBTreeMap<i32, i32> = DuplicatableBTreeMap::new();

        btree.push_back(11, 1);
        btree.push_back(11, 2);
        btree.push_back(14, 3);
        btree.push_back(18, 4);

        let (_, n1) = btree.range_mut(12..).next().unwrap();
        assert_eq!(Some(3), n1.pop_back());
    }
}
