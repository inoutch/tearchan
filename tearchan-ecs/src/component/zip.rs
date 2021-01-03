use crate::component::group::{ComponentGroup, Iter, IterMut};
use crate::component::EntityId;

pub trait ZipEntityBase {
    type Item;

    fn zip(&self, entity_id: EntityId) -> Option<Self::Item>;
}

#[derive(new)]
pub struct ZipEntityIter<'a, 'b, T, U>
where
    U: ZipEntityBase,
{
    base: Iter<'a, T>,
    other: &'b U,
}

impl<'a, 'b, T, U> Iterator for ZipEntityIter<'a, 'b, T, U>
where
    U: ZipEntityBase,
{
    type Item = (EntityId, &'a T, U::Item);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity_id, base)) = self.base.next() {
            if let Some(other_item) = self.other.zip(entity_id) {
                return Some((entity_id, base, other_item));
            }
        }
        None
    }
}

#[derive(new)]
pub struct ZipEntityIterMut<'a, 'b, T, U>
where
    U: ZipEntityBase,
{
    base: IterMut<'a, T>,
    other: &'b U,
}

impl<'a, 'b, T, U> Iterator for ZipEntityIterMut<'a, 'b, T, U>
where
    U: ZipEntityBase,
{
    type Item = (EntityId, &'a mut T, U::Item);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity_id, base)) = self.base.next() {
            if let Some(other_item) = self.other.zip(entity_id) {
                return Some((entity_id, base, other_item));
            }
        }
        None
    }
}

pub struct ZipEntity1<'a, T> {
    group: &'a ComponentGroup<T>,
}

impl<'a, T> ZipEntity1<'a, T> {
    pub fn new(group: &'a ComponentGroup<T>) -> ZipEntity1<'a, T> {
        ZipEntity1 { group }
    }
}

impl<'a, T> ZipEntityBase for ZipEntity1<'a, T> {
    type Item = &'a T;

    fn zip(&self, entity_id: u32) -> Option<&'a T> {
        self.group.get(entity_id)
    }
}

pub struct ZipEntity2<'a, 'b, T1, T2> {
    groups: (&'a ComponentGroup<T1>, &'b ComponentGroup<T2>),
}

impl<'a, 'b, T1, T2> ZipEntity2<'a, 'b, T1, T2> {
    pub fn new(
        group1: &'a ComponentGroup<T1>,
        group2: &'b ComponentGroup<T2>,
    ) -> ZipEntity2<'a, 'b, T1, T2> {
        ZipEntity2 {
            groups: (group1, group2),
        }
    }
}

impl<'a, 'b, T1, T2> ZipEntityBase for ZipEntity2<'a, 'b, T1, T2> {
    type Item = (&'a T1, &'b T2);

    fn zip(&self, entity_id: u32) -> Option<(&'a T1, &'b T2)> {
        let group1 = self.groups.0.get(entity_id)?;
        let group2 = self.groups.1.get(entity_id)?;
        Some((group1, group2))
    }
}
