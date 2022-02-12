use crate::component::EntityId;
use serde::de::{MapAccess, Unexpected, Visitor};
use serde::ser::SerializeMap;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::hash_set::Iter;
use std::collections::{BTreeSet, HashSet};
use std::fmt::Formatter;
use std::sync::{Arc, RwLock, RwLockReadGuard};

#[derive(Debug)]
struct IdManager {
    next_entity_id: EntityId,
    entity_ids: HashSet<EntityId>,
    free_entity_ids: BTreeSet<EntityId>,
}

impl Default for IdManager {
    fn default() -> Self {
        IdManager {
            next_entity_id: 1,
            entity_ids: HashSet::new(),
            free_entity_ids: BTreeSet::new(),
        }
    }
}

impl IdManager {
    pub fn gen(&mut self) -> EntityId {
        match self.free_entity_ids.pop_first() {
            None => {}
            Some(entity_id) => return entity_id,
        };
        let entity_id = self.next_entity_id;
        self.entity_ids.insert(entity_id);
        self.next_entity_id += 1;
        entity_id
    }

    pub fn free(&mut self, entity_id: EntityId) {
        if !self.entity_ids.remove(&entity_id) {
            return;
        }

        let mut update_next_entity_id = false;
        if self.next_entity_id - 1 == entity_id {
            self.next_entity_id = entity_id;
            update_next_entity_id = true;
        }

        if let Some(last) = self.free_entity_ids.last().copied() {
            if last == self.next_entity_id - 1 {
                if let Some(first) = free_continuous_entity_ids(&mut self.free_entity_ids) {
                    self.next_entity_id = first;
                    update_next_entity_id = true;
                }
            }
        }

        if !update_next_entity_id {
            self.free_entity_ids.insert(entity_id);
        }
    }

    pub fn iter(&self) -> Iter<EntityId> {
        self.entity_ids.iter()
    }
}

impl Serialize for IdManager {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("entityIds", &self.entity_ids)?;
        map.serialize_entry("freeEntityIds", &self.free_entity_ids)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for IdManager {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdManagerVisitor;
        impl<'de> Visitor<'de> for IdManagerVisitor {
            type Value = IdManager;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("idManager")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entity_ids: Option<HashSet<EntityId>> = None;
                let mut free_entity_ids: Option<BTreeSet<EntityId>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "entityIds" => {
                            entity_ids = Some(map.next_value()?);
                        }
                        "freeEntityIds" => {
                            free_entity_ids = Some(map.next_value()?);
                        }
                        _ => {}
                    }
                }
                if let Some(entity_ids) = entity_ids {
                    if let Some(free_entity_ids) = free_entity_ids {
                        let next_entity_id = entity_ids.iter().max().copied().unwrap_or(0) + 1;
                        return Ok(IdManager {
                            next_entity_id,
                            entity_ids,
                            free_entity_ids,
                        });
                    }
                }
                Err(de::Error::invalid_type(Unexpected::Map, &"unit variant"))
            }
        }
        deserializer.deserialize_map(IdManagerVisitor)
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct EntityManager(#[serde(with = "arc_rwlock_serde")] Arc<RwLock<IdManager>>);

impl EntityManager {
    pub fn new(first_id: EntityId) -> Self {
        EntityManager(Arc::new(RwLock::new(IdManager {
            next_entity_id: first_id,
            ..IdManager::default()
        })))
    }

    pub fn gen(&self) -> EntityId {
        self.0.write().unwrap().gen()
    }

    pub fn free(&self, entity_id: EntityId) {
        self.0.write().unwrap().free(entity_id);
    }

    pub fn read(&self) -> EntityManagerReader {
        EntityManagerReader(self.0.read().unwrap())
    }
}

pub struct EntityManagerReader<'a>(RwLockReadGuard<'a, IdManager>);

impl<'a> EntityManagerReader<'a> {
    pub fn iter(&self) -> Iter<EntityId> {
        self.0.iter()
    }
}

fn free_continuous_entity_ids(entity_ids: &mut BTreeSet<EntityId>) -> Option<EntityId> {
    let mut ret = None;
    while let Some(prev) = entity_ids.pop_last() {
        ret = Some(prev);
        let next = entity_ids.last();
        if Some(&(prev - 1)) != next {
            break;
        }
    }
    ret
}

mod arc_rwlock_serde {
    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use serde::{Deserialize, Serialize};
    use std::sync::{Arc, RwLock};

    pub fn serialize<S, T>(val: &Arc<RwLock<T>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        T::serialize(&*val.read().unwrap(), s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Arc<RwLock<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Ok(Arc::new(RwLock::new(T::deserialize(d)?)))
    }
}

#[cfg(test)]
mod test {
    use crate::entity::manager::{free_continuous_entity_ids, EntityManager};
    use std::collections::BTreeSet;

    #[test]
    fn test_free_continuous_entity_ids() {
        let mut ids = BTreeSet::new();
        ids.insert(4);
        ids.insert(6);
        ids.insert(5);
        ids.insert(2);
        ids.insert(1);

        assert_eq!(free_continuous_entity_ids(&mut ids), Some(4));
    }

    #[test]
    fn test_free_continuous_entity_ids_none() {
        let mut ids = BTreeSet::new();
        assert_eq!(free_continuous_entity_ids(&mut ids), None);
    }

    #[test]
    fn test_free_continuous_entity_ids_one() {
        let mut ids = BTreeSet::new();
        ids.insert(6);
        assert_eq!(free_continuous_entity_ids(&mut ids), Some(6));
    }

    #[test]
    fn test_entity_manager_gen() {
        let entity_manager = EntityManager::default();
        let entity1 = entity_manager.gen();
        let entity2 = entity_manager.gen();
        let entity3 = entity_manager.gen();
        let entity4 = entity_manager.gen();
        let entity5 = entity_manager.gen();

        assert_eq!(entity1, 1);
        assert_eq!(entity2, 2);
        assert_eq!(entity3, 3);
        assert_eq!(entity4, 4);
        assert_eq!(entity5, 5);

        let mut entity_ids = entity_manager.read().iter().copied().collect::<Vec<_>>();
        entity_ids.sort();
        assert_eq!(entity_ids, vec![1, 2, 3, 4, 5]);

        entity_manager.free(entity2);
        assert_eq!(entity_manager.0.read().unwrap().next_entity_id, 6);

        let entity6 = entity_manager.gen();
        assert_eq!(entity6, 2);

        entity_manager.free(entity5);
        assert_eq!(entity_manager.0.read().unwrap().next_entity_id, 5);
        assert_eq!(
            entity_manager.0.read().unwrap().free_entity_ids.last(),
            None
        );

        entity_manager.free(entity3);
        assert_eq!(entity_manager.0.read().unwrap().next_entity_id, 5);
        assert_eq!(
            entity_manager.0.read().unwrap().free_entity_ids.last(),
            Some(&3)
        );

        entity_manager.free(entity4);
        assert_eq!(entity_manager.0.read().unwrap().next_entity_id, 3);
    }

    #[test]
    fn test_entity_manager_serialization() {
        let entity_manager = EntityManager::default();

        let _entity1 = entity_manager.gen();
        let entity2 = entity_manager.gen();
        let entity3 = entity_manager.gen();
        let entity4 = entity_manager.gen();
        let entity5 = entity_manager.gen();

        entity_manager.free(entity2);
        assert_eq!(entity_manager.0.read().unwrap().next_entity_id, 6);

        let entity6 = entity_manager.gen();
        assert_eq!(entity6, 2);

        entity_manager.free(entity5);
        assert_eq!(entity_manager.0.read().unwrap().next_entity_id, 5);
        assert_eq!(
            entity_manager.0.read().unwrap().free_entity_ids.last(),
            None
        );

        let json = serde_json::to_string(&entity_manager).unwrap();
        let entity_manager: EntityManager = serde_json::from_str(&json).unwrap();

        entity_manager.free(entity3);
        assert_eq!(entity_manager.0.read().unwrap().next_entity_id, 5);
        assert_eq!(
            entity_manager.0.read().unwrap().free_entity_ids.last(),
            Some(&3)
        );

        entity_manager.free(entity4);
        assert_eq!(entity_manager.0.read().unwrap().next_entity_id, 3);
    }
}
