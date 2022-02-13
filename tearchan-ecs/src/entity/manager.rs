use crate::component::EntityId;
use serde::de::{MapAccess, Unexpected, Visitor};
use serde::ser::SerializeMap;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::btree_set::Iter;
use std::collections::BTreeSet;
use std::fmt::Formatter;
use std::sync::{Arc, RwLock, RwLockReadGuard};

#[derive(Debug)]
struct IdManager {
    entity_ids: BTreeSet<EntityId>,
    incremental_id_manager: tearchan_util::id_manager::IdManager<EntityId>,
}

impl Default for IdManager {
    fn default() -> Self {
        IdManager {
            entity_ids: BTreeSet::new(),
            incremental_id_manager: tearchan_util::id_manager::IdManager::new(1, |id| *id + 1),
        }
    }
}

impl IdManager {
    pub fn gen(&mut self) -> EntityId {
        let entity_id = self.incremental_id_manager.gen();
        self.entity_ids.insert(entity_id);
        entity_id
    }

    pub fn free(&mut self, entity_id: EntityId) {
        self.entity_ids.remove(&entity_id);
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
        map.serialize_entry("first", &*self.incremental_id_manager.current())?;
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
                let mut entity_ids: Option<BTreeSet<EntityId>> = None;
                let mut first: Option<EntityId> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "entityIds" => {
                            entity_ids = Some(map.next_value()?);
                        }
                        "first" => {
                            first = Some(map.next_value()?);
                        }
                        _ => {}
                    }
                }
                if let Some(entity_ids) = entity_ids {
                    if let Some(first) = first {
                        return Ok(IdManager {
                            entity_ids,
                            incremental_id_manager: tearchan_util::id_manager::IdManager::new(
                                first,
                                |id| *id + 1,
                            ),
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
            incremental_id_manager: tearchan_util::id_manager::IdManager::new(first_id, |id| {
                *id + 1
            }),
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
    use crate::component::EntityId;
    use crate::entity::manager::IdManager;

    #[test]
    fn test_iter() {
        let mut id_manager = IdManager::default();
        let id_0 = id_manager.gen();
        let id_1 = id_manager.gen();
        let id_2 = id_manager.gen();

        id_manager.free(id_0);
        id_manager.free(id_1);
        id_manager.free(id_2);

        assert_eq!(
            id_manager.iter().copied().collect::<Vec<EntityId>>().len(),
            0
        );

        let id_3 = id_manager.gen();
        assert_eq!(id_manager.iter().copied().collect::<Vec<_>>(), vec![id_3]);
    }

    #[test]
    fn test_serialization() {
        let mut id_manager = IdManager::default();
        let id_0 = id_manager.gen();
        let id_1 = id_manager.gen();
        let id_2 = id_manager.gen();

        id_manager.free(id_0);

        let json = serde_json::to_string(&id_manager).unwrap();
        let mut id_manager_restored: IdManager = serde_json::from_str(&json).unwrap();

        assert_eq!(
            id_manager_restored.iter().copied().collect::<Vec<_>>(),
            vec![id_1, id_2]
        );

        id_manager_restored.free(id_2);

        assert_eq!(
            id_manager_restored.iter().copied().collect::<Vec<_>>(),
            vec![id_1]
        );

        assert_eq!(id_manager_restored.gen(), 4);
    }
}
