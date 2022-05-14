use crate::component::EntityId;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::btree_set::Iter;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
struct IdManager {
    entity_ids: BTreeSet<EntityId>,
    vacated_entities: BTreeSet<EntityId>,
    incremental_id_manager: tearchan_util::id_manager::IdManager<EntityId>,
}

impl Default for IdManager {
    fn default() -> Self {
        IdManager {
            entity_ids: BTreeSet::new(),
            vacated_entities: Default::default(),
            incremental_id_manager: tearchan_util::id_manager::IdManager::new(1, |id| *id + 1),
        }
    }
}

impl IdManager {
    pub fn gen(&mut self) -> EntityId {
        let entity_id = self.incremental_id_manager.gen();
        self.entity_ids.insert(entity_id);
        self.vacated_entities.insert(entity_id);
        entity_id
    }

    pub fn next(&self) -> EntityId {
        *self.incremental_id_manager.current()
    }

    pub fn free(&mut self, entity_id: EntityId) {
        self.vacated_entities.remove(&entity_id);
        self.entity_ids.remove(&entity_id);
    }

    pub fn iter(&self) -> Iter<EntityId> {
        self.entity_ids.iter()
    }

    pub fn pull_vacated_entities(&mut self) -> BTreeSet<EntityId> {
        std::mem::take(&mut self.vacated_entities)
    }
}

#[derive(Default, Debug)]
pub struct EntityManager(Arc<RwLock<IdManager>>);

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

    pub fn contains(&self, entity_id: EntityId) -> bool {
        self.0.read().unwrap().entity_ids.contains(&entity_id)
    }

    pub fn load_data(&self, data: EntityManagerData) -> EntityRemapperToken {
        let mut mapping: HashMap<EntityId, EntityId> = HashMap::new(); // key = from, value = to
        for entity_id in data.entity_ids {
            mapping.insert(entity_id, self.gen());
        }
        EntityRemapperToken::new(Some(mapping))
    }

    pub fn to_data(&self) -> EntityManagerData {
        EntityManagerData {
            entity_ids: self.0.read().unwrap().entity_ids.clone(),
        }
    }

    pub fn begin(&self) -> EntityToken {
        let guard = self.0.write().unwrap();
        EntityToken {
            entity_id: guard.next(),
            guard_id_manager: guard,
        }
    }

    pub fn pull_vacated_entities(&self) -> BTreeSet<EntityId> {
        self.0.write().unwrap().pull_vacated_entities()
    }
}

pub struct EntityToken<'a> {
    entity_id: EntityId,
    guard_id_manager: RwLockWriteGuard<'a, IdManager>,
}

impl<'a> EntityToken<'a> {
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn commit(mut self) -> EntityId {
        self.guard_id_manager.gen()
    }
}

pub struct EntityManagerReader<'a>(RwLockReadGuard<'a, IdManager>);

impl<'a> EntityManagerReader<'a> {
    pub fn iter(&self) -> Iter<EntityId> {
        self.0.iter()
    }
}

#[derive(Serialize, Deserialize)]
pub struct EntityManagerData {
    entity_ids: BTreeSet<EntityId>,
}

#[derive(Default)]
pub struct EntityRemapper {
    mapping: Mutex<Option<HashMap<EntityId, EntityId>>>,
}

impl EntityRemapper {
    pub fn remap(&self, entity_id: EntityId) -> EntityId {
        let mapping = self.mapping.lock().unwrap();
        if let Some(mapping) = mapping.as_ref() {
            return *mapping.get(&entity_id).unwrap_or(&entity_id);
        }
        entity_id
    }

    pub fn lock(&self) -> EntityRemapperToken {
        EntityRemapperToken {
            _guard: ENTITY_REMAPPER_WRITE_LOCK.lock().unwrap(),
        }
    }
}

pub static ENTITY_REMAPPER: Lazy<EntityRemapper> = Lazy::new(EntityRemapper::default);
static ENTITY_REMAPPER_WRITE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub struct EntityRemapperToken<'a> {
    _guard: MutexGuard<'a, ()>,
}

impl<'a> EntityRemapperToken<'a> {
    fn new(mapping: Option<HashMap<EntityId, EntityId>>) -> Self {
        let guard = ENTITY_REMAPPER_WRITE_LOCK.lock().unwrap();
        *ENTITY_REMAPPER.mapping.lock().unwrap() = mapping;
        Self { _guard: guard }
    }
}

impl<'a> Drop for EntityRemapperToken<'a> {
    fn drop(&mut self) {
        *ENTITY_REMAPPER.mapping.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod test {
    use crate::entity::manager::{EntityManager, EntityManagerData, IdManager, ENTITY_REMAPPER};
    use std::collections::BTreeSet;
    use std::time::Duration;

    #[test]
    fn test_iter() {
        let mut id_manager = IdManager::default();
        let id_0 = id_manager.gen();
        let id_1 = id_manager.gen();
        let id_2 = id_manager.gen();

        assert_eq!(
            id_manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![id_0, id_1, id_2]
        );
        assert_eq!(id_manager.pull_vacated_entities().len(), 0);

        id_manager.free(id_0);
        id_manager.free(id_1);
        id_manager.free(id_2);

        assert_eq!(id_manager.iter().copied().count(), 0);

        let id_3 = id_manager.gen();
        assert_eq!(id_manager.iter().copied().collect::<Vec<_>>(), vec![id_3]);
    }

    #[test]
    fn test_serialization_and_remap() {
        let entity_manager = EntityManager::default();
        let entity_id1 = entity_manager.gen();
        let entity_id2 = entity_manager.gen();
        let entity_id3 = entity_manager.gen();
        let entity_id4 = 7;

        let json = serde_json::to_string(&entity_manager.to_data()).unwrap();

        let data: EntityManagerData = serde_json::from_str(&json).unwrap();
        {
            let _token = entity_manager.load_data(data);
            assert_ne!(entity_id1, ENTITY_REMAPPER.remap(entity_id1));
            assert_ne!(entity_id2, ENTITY_REMAPPER.remap(entity_id2));
            assert_ne!(entity_id3, ENTITY_REMAPPER.remap(entity_id3));
            assert_eq!(entity_id4, ENTITY_REMAPPER.remap(entity_id4));
        }
        assert_eq!(entity_id1, ENTITY_REMAPPER.remap(entity_id1));
    }

    #[test]
    fn test_commitment() {
        let entity_manager = EntityManager::default();
        let entity_id0 = {
            let token = entity_manager.begin();
            let entity_id = token.entity_id();
            token.commit();
            entity_id
        };
        {
            let token = entity_manager.begin();
            let _entity_id = token.entity_id();
        };
        let entity_id1 = {
            let token = entity_manager.begin();
            let entity_id = token.entity_id();
            token.commit();
            entity_id
        };
        {
            let token = entity_manager.begin();
            let _entity_id = token.entity_id();
        };
        assert_eq!(
            entity_manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![entity_id0, entity_id1]
        );
        assert_eq!(
            entity_manager.read().iter().collect::<Vec<_>>(),
            vec![&entity_id0, &entity_id1]
        );
    }

    #[test]
    fn test_sync() {
        let h0 = std::thread::spawn(|| {
            let entity_manager = EntityManager::default();
            entity_manager.gen(); // 1
            entity_manager.gen(); // 2

            let mut entity_ids = BTreeSet::default();
            entity_ids.insert(1);
            entity_ids.insert(2);

            let _token = entity_manager.load_data(EntityManagerData { entity_ids });
            std::thread::sleep(Duration::from_millis(500));

            assert_eq!(
                ENTITY_REMAPPER
                    .mapping
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .get(&1),
                Some(&3)
            );
            assert_eq!(
                ENTITY_REMAPPER
                    .mapping
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .get(&2),
                Some(&4)
            );
        });
        let h1 = std::thread::spawn(|| {
            let _lock = ENTITY_REMAPPER.lock();
            std::thread::sleep(Duration::from_millis(200));

            assert_eq!(ENTITY_REMAPPER.remap(1), 1);
            assert_eq!(ENTITY_REMAPPER.remap(2), 2);
        });
        h0.join().unwrap();
        h1.join().unwrap();
    }
}
