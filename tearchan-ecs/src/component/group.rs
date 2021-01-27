use crate::component::zip::{ZipEntityBase, ZipEntityIter, ZipEntityIterMut};
use crate::component::{Component, EntityId};
use std::collections::HashMap;

pub type ComponentIndex = usize;

pub struct ComponentGroup<T> {
    indices: HashMap<EntityId, ComponentIndex>,
    components: Vec<Component<T>>,
}

impl<T> Default for ComponentGroup<T> {
    fn default() -> Self {
        Self {
            indices: HashMap::new(),
            components: Vec::new(),
        }
    }
}

impl<T> ComponentGroup<T> {
    pub fn push(&mut self, entity_id: EntityId, inner: T) {
        debug_assert!(!self.indices.contains_key(&entity_id));
        self.components.push(Component::new(entity_id, inner));
        self.indices.insert(entity_id, self.components.len() - 1);
    }

    pub fn remove(&mut self, entity_id: EntityId) -> Option<Component<T>> {
        let index = self.indices.remove(&entity_id)?;
        let component = self.components.remove(index);

        for i in self.indices.values_mut() {
            if *i > index {
                *i -= 1;
            }
        }
        Some(component)
    }

    pub fn get(&self, entity_id: EntityId) -> Option<&T> {
        let index = self.indices.get(&entity_id)?;
        self.components
            .get(*index)
            .map(|component| component.inner())
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.components.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            iter: self.components.iter_mut(),
        }
    }
}

pub struct Iter<'a, T> {
    iter: std::slice::Iter<'a, Component<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (EntityId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        Some((next.entity_id(), next.inner()))
    }
}

impl<'a, T> Iter<'a, T> {
    pub fn zip_entities<'b, U>(self, other: &'b U) -> ZipEntityIter<'a, 'b, T, U>
    where
        U: ZipEntityBase,
    {
        ZipEntityIter::new(self, other)
    }
}

pub struct IterMut<'a, T> {
    iter: std::slice::IterMut<'a, Component<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (EntityId, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        Some((next.entity_id(), next.inner_mut()))
    }
}

impl<'a, T> IterMut<'a, T> {
    pub fn zip_entities_mut<'b, U>(self, other: &'b U) -> ZipEntityIterMut<'a, 'b, T, U>
    where
        U: ZipEntityBase,
    {
        ZipEntityIterMut::new(self, other)
    }
}

#[cfg(test)]
mod test {
    use crate::component::group::ComponentGroup;
    use crate::component::zip::ZipEntity2;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct Inner(&'static str);

    #[test]
    fn test_len() {
        let mut group: ComponentGroup<Inner> = ComponentGroup::default();
        assert_eq!(group.len(), 0);

        group.push(1, Inner("ex1"));
        group.push(2, Inner("ex2"));
        group.push(3, Inner("ex3"));
        group.push(4, Inner("ex4"));

        assert_eq!(group.len(), 4);
    }

    #[test]
    fn test_is_empty() {
        let mut group: ComponentGroup<Inner> = ComponentGroup::default();
        assert!(group.is_empty());

        group.push(1, Inner("ex1"));

        assert!(!group.is_empty());
    }

    #[test]
    fn test_remove() {
        let mut group: ComponentGroup<Inner> = ComponentGroup::default();
        group.push(1, Inner("ex1"));
        group.push(2, Inner("ex2"));
        group.push(3, Inner("ex3"));
        group.push(4, Inner("ex4"));

        assert_eq!(group.len(), 4);

        assert!(group.remove(2).is_some());

        assert_eq!(group.len(), 3);

        assert!(group.remove(2).is_none());
        assert!(group.remove(5).is_none());
    }

    #[test]
    fn test_iter() {
        let mut group: ComponentGroup<Inner> = ComponentGroup::default();
        group.push(1, Inner("ex1"));
        group.push(2, Inner("ex2"));
        group.push(3, Inner("ex3"));
        group.push(4, Inner("ex4"));

        let mut iter = group.iter();
        assert_eq!(iter.next().unwrap().1 .0, "ex1");
        assert_eq!(iter.next().unwrap().1 .0, "ex2");
        assert_eq!(iter.next().unwrap().1 .0, "ex3");
        assert_eq!(iter.next().unwrap().1 .0, "ex4");
    }

    #[test]
    fn test_zip() {
        let mut group1: ComponentGroup<Inner> = ComponentGroup::default();
        let mut group2: ComponentGroup<Inner> = ComponentGroup::default();
        let mut group3: ComponentGroup<Inner> = ComponentGroup::default();

        group1.push(1, Inner("ex1"));
        group1.push(2, Inner("ex2"));

        group2.push(1, Inner("ex1"));
        group2.push(2, Inner("ex2"));

        group3.push(1, Inner("ex1"));
        group3.push(2, Inner("ex2"));

        let counter = Rc::new(RefCell::new(0));
        let counter_ref = &counter;
        let group1 = group1;
        let group2 = group2;
        let group3 = group3;
        let entities = ZipEntity2::new(&group2, &group3);
        group1.iter().zip_entities(&entities).for_each(
            move |(_, component1, (component2, component3))| {
                let counter = Rc::clone(counter_ref);
                *counter.borrow_mut() += 1;
                assert_eq!(component1.0, component2.0);
                assert_eq!(component2.0, component3.0);
            },
        );

        assert_eq!(*counter.borrow(), 2);
    }
}
