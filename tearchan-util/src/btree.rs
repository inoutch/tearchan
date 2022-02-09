use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::btree_map::{Iter, IterMut, Range, RangeMut};
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
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicatableBTreeMap<K, V>
where
    K: Ord,
{
    btree: BTreeMap<K, VecDeque<V>>,
}

impl<K, V> Default for DuplicatableBTreeMap<K, V>
where
    K: Ord,
{
    fn default() -> DuplicatableBTreeMap<K, V> {
        DuplicatableBTreeMap {
            btree: BTreeMap::new(),
        }
    }
}

impl<K, V> DuplicatableBTreeMap<K, V>
where
    K: Ord,
{
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

    pub fn pop_first_back(&mut self) -> Option<V>
    where
        K: Ord,
    {
        loop {
            return match self.btree.pop_first() {
                None => None,
                Some((key, mut queue)) => {
                    let ret = match queue.pop_front() {
                        None => continue,
                        Some(x) => x,
                    };
                    if !queue.is_empty() {
                        self.btree.insert(key, queue);
                    }
                    Some(ret)
                }
            };
        }
    }

    pub fn pop_last_back(&mut self) -> Option<V>
    where
        K: Ord,
    {
        match self.btree.pop_last() {
            None => None,
            Some((key, mut queue)) => {
                let ret = match queue.pop_front() {
                    None => panic!("Illegal state"),
                    Some(x) => x,
                };
                if !queue.is_empty() {
                    self.btree.insert(key, queue);
                }
                Some(ret)
            }
        }
    }

    pub fn remove(&mut self, key: &K, value: &V) -> Option<V>
    where
        V: Eq,
    {
        let values = self.btree.get_mut(key)?;
        let index = values.iter().position(|v| v == value)?;
        values.remove(index)
    }

    pub fn first_key_value(&self) -> Option<(&K, &V)> {
        self.btree
            .first_key_value()
            .and_then(|(key, values)| values.front().map(|value| (key, value)))
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
    pub fn iter_mut(&mut self) -> IterMut<'_, K, VecDeque<V>> {
        self.btree.iter_mut()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.btree.clear();
    }
}

impl<K, V> From<DuplicatableBTreeMap<K, V>> for Vec<V>
where
    K: Ord,
{
    fn from(mut map: DuplicatableBTreeMap<K, V>) -> Self {
        let mut ret = vec![];
        while let Some(x) = map.pop_first_back() {
            ret.push(x);
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use crate::btree::DuplicatableBTreeMap;

    #[test]
    pub fn test_btree() {
        let mut btree: DuplicatableBTreeMap<i32, i32> = DuplicatableBTreeMap::default();

        btree.push_back(11, 1);
        btree.push_back(11, 2);
        btree.push_back(14, 3);
        btree.push_back(18, 4);

        let (_, n1) = btree.range_mut(12..).next().unwrap();
        assert_eq!(Some(3), n1.pop_back());
    }
}
