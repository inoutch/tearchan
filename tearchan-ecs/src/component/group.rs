use crate::component::zip::{ZipEntityBase, ZipEntityIter, ZipEntityIterMut};
use crate::component::{Component, EntityId};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Enumerate;
use std::marker::PhantomData;

pub type ComponentIndex = usize;

pub struct ComponentGroup<T> {
    indices: HashMap<EntityId, ComponentIndex>,
    components: Vec<Component<T>>,
    pending_indices: Vec<ComponentIndex>,
}

impl<T> Default for ComponentGroup<T> {
    fn default() -> Self {
        Self {
            indices: HashMap::new(),
            components: Vec::new(),
            pending_indices: Vec::new(),
        }
    }
}

impl<T> ComponentGroup<T> {
    pub fn push(&mut self, entity_id: EntityId, inner: T) {
        debug_assert!(!self.indices.contains_key(&entity_id));

        let component = Component::new(entity_id, inner);
        if let Some(index) = self.pending_indices.pop() {
            *self.components.get_mut(index).unwrap() = component;
            self.indices.insert(entity_id, index);
            return;
        }

        self.components.push(component);
        self.indices.insert(entity_id, self.components.len() - 1);
    }

    pub fn remove(&mut self, entity_id: EntityId) -> Option<&T> {
        let index = match self.indices.remove(&entity_id) {
            None => return None,
            Some(index) => index,
        };
        self.pending_indices.push(index);
        self.components
            .get(index)
            .map(|component| component.inner())
    }

    pub fn remove_all(&mut self) {
        self.indices.clear();
        self.pending_indices.clear();
        self.components.clear();
    }

    pub fn get(&self, entity_id: EntityId) -> Option<&T> {
        let index = self.indices.get(&entity_id)?;
        self.components
            .get(*index)
            .map(|component| component.inner())
    }

    pub fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut T> {
        let index = self.indices.get(&entity_id)?;
        self.components
            .get_mut(*index)
            .map(|component| component.inner_mut())
    }

    pub fn get_with_err(&self, entity_id: EntityId) -> Result<&T, ComponentGroupError> {
        self.get(entity_id)
            .ok_or_else(|| ComponentGroupError::NotFoundEntity { id: entity_id })
    }

    pub fn get_mut_with_err(&mut self, entity_id: EntityId) -> Result<&mut T, ComponentGroupError> {
        self.get_mut(entity_id)
            .ok_or_else(|| ComponentGroupError::NotFoundEntity { id: entity_id })
    }

    pub fn entity(&self, entity_id: EntityId) -> &T {
        self.get(entity_id)
            .unwrap_or_else(|| panic!("The entity of {} id is not found", entity_id))
    }

    pub fn entity_mut(&mut self, entity_id: EntityId) -> &mut T {
        self.get_mut(entity_id)
            .unwrap_or_else(|| panic!("The entity of {} id is not found", entity_id))
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.components.iter().enumerate(),
            indices: &self.indices,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            iter_mut: self.components.iter_mut().enumerate(),
            indices: &self.indices,
        }
    }

    pub fn debug(&self) -> ComponentGroupDebug<T>
    where
        T: Debug,
    {
        ComponentGroupDebug(&self.components)
    }
}

impl<T> Serialize for ComponentGroup<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let components = self
            .components
            .iter()
            .filter(|component| self.indices.contains_key(&component.entity_id))
            .collect::<Vec<_>>();

        let mut seq = serializer.serialize_map(Some(components.len()))?;
        for component in components {
            seq.serialize_entry(&component.entity_id(), &component.inner)?;
        }
        seq.end()
    }
}

impl<'de, T> Deserialize<'de> for ComponentGroup<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct ComponentGroupVisitor<T>(PhantomData<T>);
        impl<'de, T> Visitor<'de> for ComponentGroupVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = ComponentGroup<T>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("componentGroup")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
            where
                A: MapAccess<'de>,
            {
                let mut components = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((key, value)) = map.next_entry()? {
                    components.push(Component::new(key, value));
                }

                let indices = components
                    .iter()
                    .enumerate()
                    .map(|(index, component): (usize, &Component<T>)| {
                        (component.entity_id(), index)
                    })
                    .collect();
                Ok(ComponentGroup {
                    indices,
                    components,
                    pending_indices: Vec::new(),
                })
            }
        }
        deserializer.deserialize_map(ComponentGroupVisitor(PhantomData))
    }
}

pub struct Iter<'a, T> {
    iter: Enumerate<std::slice::Iter<'a, Component<T>>>,
    indices: &'a HashMap<EntityId, ComponentIndex>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (EntityId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (index, next) = self.iter.next()?;
            if self
                .indices
                .get(&next.entity_id)
                .map(|exist_index| exist_index == &index)
                .unwrap_or(false)
            {
                return Some((next.entity_id(), next.inner()));
            }
        }
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
    iter_mut: Enumerate<std::slice::IterMut<'a, Component<T>>>,
    indices: &'a HashMap<EntityId, ComponentIndex>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (EntityId, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (index, next) = self.iter_mut.next()?;
            if self
                .indices
                .get(&next.entity_id)
                .map(|exist_index| exist_index == &index)
                .unwrap_or(false)
            {
                return Some((next.entity_id(), next.inner_mut()));
            }
        }
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

#[derive(Debug)]
pub enum ComponentGroupError {
    NotFoundEntity { id: EntityId },
}

impl Error for ComponentGroupError {}

impl Display for ComponentGroupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentGroupError::NotFoundEntity { id } => {
                write!(f, "The entity of {} id is not found", id)
            }
        }
    }
}

#[derive(Debug)]
pub struct ComponentGroupDebug<'a, T: Debug>(&'a Vec<Component<T>>);

#[cfg(test)]
mod test {
    use crate::component::group::ComponentGroup;
    use crate::component::zip::ZipEntity2;
    use serde::{Deserialize, Serialize};
    use std::cell::RefCell;
    use std::rc::Rc;

    struct Inner(&'static str);

    #[derive(Serialize, Deserialize)]
    struct SerializationInner(String);

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

    #[test]
    fn test_serialization() {
        let mut group = ComponentGroup::default();
        group.push(0, SerializationInner("entity 0".to_string()));
        group.push(1, SerializationInner("entity 1".to_string()));
        group.push(2, SerializationInner("entity 2".to_string()));
        group.push(3, SerializationInner("entity 3".to_string()));
        group.remove(3);

        let str = serde_json::to_string(&group).unwrap();
        let group: ComponentGroup<SerializationInner> = serde_json::from_str(&str).unwrap();
        assert_eq!(group.get(0).as_ref().unwrap().0, "entity 0");
        assert_eq!(group.get(1).as_ref().unwrap().0, "entity 1");
        assert_eq!(group.get(2).as_ref().unwrap().0, "entity 2");
        assert!(group.get(3).is_none());
    }

    #[test]
    fn test_iter_with_removing() {
        let mut group = ComponentGroup::default();
        group.push(0, 10);
        group.push(1, 11);
        group.push(2, 12);
        group.push(3, 13);
        group.remove(3);

        let mut iter = group.iter();
        assert_eq!(iter.next(), Some((0, &10)));
        assert_eq!(iter.next(), Some((1, &11)));
        assert_eq!(iter.next(), Some((2, &12)));
        assert_eq!(iter.next(), None);

        let mut iter = group.iter_mut();
        assert_eq!(iter.next(), Some((0, &mut 10)));
        assert_eq!(iter.next(), Some((1, &mut 11)));
        assert_eq!(iter.next(), Some((2, &mut 12)));
        assert_eq!(iter.next(), None);
    }
}
