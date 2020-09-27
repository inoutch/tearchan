use crate::action::action_context::ActionContext;
use crate::action::action_store::ActionStore;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use tearchan_core::game::object::GameObjectId;
use tearchan_utility::btree::DuplicatableBTreeMap;

#[derive(Debug, Clone)]
pub enum ActionResult<TCommonActionStore> {
    Start {
        store: Rc<ActionStore<TCommonActionStore>>,
    },
    Update {
        store: Rc<ActionStore<TCommonActionStore>>,
        current_time: u64,
    },
    End {
        store: Rc<ActionStore<TCommonActionStore>>,
    },
}

impl<TCommonActionStore> ActionResult<TCommonActionStore> {
    pub fn object_id(&self) -> GameObjectId {
        match self {
            ActionResult::Start { store } => store.object_id,
            ActionResult::Update { store, .. } => store.object_id,
            ActionResult::End { store } => store.object_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionStoreQueueItem<TCommonActionStore> {
    object_id: GameObjectId,
    store: Rc<ActionStore<TCommonActionStore>>,
}

#[derive(Serialize, Deserialize)]
pub struct ActionManagerSnapshot<TCommonActionStore> {
    waiting_stores: DuplicatableBTreeMap<u64, ActionStoreQueueItem<TCommonActionStore>>,
    started_stores: DuplicatableBTreeMap<u64, ActionStoreQueueItem<TCommonActionStore>>,
    current_time: u64,
}

pub struct ActionManager<TCommonActionStore> {
    contexts: HashMap<GameObjectId, ActionContext>,
    snapshot: ActionManagerSnapshot<TCommonActionStore>,
}

impl<TCommonActionStore: Debug> ActionManager<TCommonActionStore> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ActionManager {
            contexts: HashMap::new(),
            snapshot: ActionManagerSnapshot {
                waiting_stores: DuplicatableBTreeMap::new(),
                started_stores: DuplicatableBTreeMap::new(),
                current_time: 0,
            },
        }
    }

    pub fn new_with_snapshot(snapshot: ActionManagerSnapshot<TCommonActionStore>) -> Self {
        let mut contexts = HashMap::new();
        for (_, stores) in snapshot.waiting_stores.iter() {
            for store in stores {
                insert_to_context(&mut contexts, store);
            }
        }
        for (_, stores) in snapshot.started_stores.iter() {
            for store in stores {
                insert_to_context(&mut contexts, store);
            }
        }

        ActionManager { contexts, snapshot }
    }

    pub fn update(&mut self, delta: u64) {
        self.snapshot.current_time += delta;
    }

    pub fn run(&mut self, object_id: GameObjectId, store: TCommonActionStore, duration: u64) {
        let context = match self.contexts.get_mut(&object_id) {
            Some(x) => x,
            None => {
                self.contexts.insert(
                    object_id,
                    ActionContext {
                        last_time: self.snapshot.current_time,
                        store_size: 0,
                    },
                );
                self.contexts.get_mut(&object_id).unwrap()
            }
        };
        let start_time = context.last_time;
        let end_time = start_time + duration;
        let store = Rc::new(ActionStore::new(store, object_id, start_time, end_time));
        self.snapshot.waiting_stores.push_back(
            start_time,
            ActionStoreQueueItem {
                store: Rc::clone(&store),
                object_id,
            },
        );
        context.last_time = end_time;
        context.store_size += 1;
    }

    pub fn pull(&mut self) -> DuplicatableBTreeMap<u64, ActionResult<TCommonActionStore>> {
        let mut ret = DuplicatableBTreeMap::new();
        while let Some(store) = self.snapshot.waiting_stores.pop_first_back() {
            if store.store.start_time > self.snapshot.current_time {
                self.snapshot
                    .waiting_stores
                    .push_front(store.store.start_time, store);
                break;
            }
            let object_id = store.object_id;
            let start_time = store.store.start_time;
            let end_time = store.store.end_time;
            let inner_store = Rc::clone(&store.store);
            if end_time > self.snapshot.current_time {
                self.snapshot.started_stores.push_front(end_time, store);
                ret.push_back(start_time, ActionResult::Start { store: inner_store });
            } else {
                ret.push_back(
                    start_time,
                    ActionResult::Start {
                        store: Rc::clone(&inner_store),
                    },
                );
                ret.push_back(end_time, ActionResult::End { store: inner_store });

                let context = self.contexts.get_mut(&object_id).unwrap();
                context.store_size -= 1;
                if context.store_size == 0 {
                    self.contexts.remove(&object_id);
                }
            }
        }

        while let Some(store) = self.snapshot.started_stores.pop_first_back() {
            if store.store.end_time > self.snapshot.current_time {
                self.snapshot
                    .started_stores
                    .push_front(store.store.end_time, store);
                break;
            }

            let object_id = store.store.object_id;
            ret.push_front(
                store.store.end_time,
                ActionResult::End { store: store.store },
            );

            let context = self.contexts.get_mut(&object_id).unwrap();
            context.store_size -= 1;
            if context.store_size == 0 {
                self.contexts.remove(&object_id);
            }
        }

        for (_, started_stores) in self.snapshot.started_stores.iter() {
            for started_store in started_stores.iter() {
                ret.push_back(
                    self.snapshot.current_time,
                    ActionResult::Update {
                        store: Rc::clone(&started_store.store),
                        current_time: self.snapshot.current_time,
                    },
                )
            }
        }
        ret
    }

    pub fn is_running(&self, object_id: &GameObjectId) -> bool {
        self.contexts.contains_key(object_id)
    }

    pub fn action_size(&self, object_id: &GameObjectId) -> usize {
        self.contexts.get(object_id).map_or(0, |x| x.store_size)
    }

    pub fn load(&mut self, snapshot: ActionManagerSnapshot<TCommonActionStore>) {
        self.snapshot = snapshot;
        self.update_contexts();
    }

    pub fn snapshot(&self) -> &ActionManagerSnapshot<TCommonActionStore> {
        &self.snapshot
    }

    fn update_contexts(&mut self) {
        self.contexts.clear();
        update_contexts_by_stores(&mut self.contexts, &self.snapshot.started_stores);
        update_contexts_by_stores(&mut self.contexts, &self.snapshot.waiting_stores);
    }
}

fn insert_to_context<TCommonActionStore>(
    contexts: &mut HashMap<GameObjectId, ActionContext>,
    store: &ActionStoreQueueItem<TCommonActionStore>,
) {
    let object_id = store.object_id;
    if let Some(c) = contexts.get_mut(&object_id) {
        let last_time = max(store.store.end_time, c.last_time);
        c.last_time = last_time;
        c.store_size += 1;
    } else {
        contexts.insert(
            object_id,
            ActionContext {
                last_time: store.store.end_time,
                store_size: 1,
            },
        );
    }
}

fn update_contexts_by_stores<TCommonActionStore>(
    contexts: &mut HashMap<GameObjectId, ActionContext>,
    stores: &DuplicatableBTreeMap<u64, ActionStoreQueueItem<TCommonActionStore>>,
) {
    for (_, waiting_stores) in stores.iter() {
        for started_store in waiting_stores {
            match contexts.get_mut(&started_store.object_id) {
                Some(context) => {
                    context.last_time = context.last_time.max(started_store.store.end_time);
                    context.store_size += 1;
                }
                None => {
                    contexts.insert(
                        started_store.object_id,
                        ActionContext {
                            last_time: started_store.store.end_time,
                            store_size: 1,
                        },
                    );
                }
            };
        }
    }
}

#[cfg(test)]
mod test {
    use crate::action::action_manager::{ActionManager, ActionManagerSnapshot, ActionResult};
    use serde::{Deserialize, Serialize};
    use tearchan_core::game::object::GameObjectId;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum CommonActionStore {
        Action1,
        Action2,
        Action3,
        Action4,
    }

    #[test]
    fn test_run() {
        let object1_id: GameObjectId = 0;
        let object2_id: GameObjectId = 1;
        let mut action_manager: ActionManager<CommonActionStore> = ActionManager::new();
        action_manager.run(object1_id, CommonActionStore::Action1, 10);
        action_manager.run(object1_id, CommonActionStore::Action2, 20);
        action_manager.run(object1_id, CommonActionStore::Action3, 40);
        action_manager.run(object1_id, CommonActionStore::Action4, 0);

        action_manager.run(object2_id, CommonActionStore::Action2, 30);
        action_manager.run(object2_id, CommonActionStore::Action3, 10);
        action_manager.run(object2_id, CommonActionStore::Action4, 60);
        action_manager.run(object2_id, CommonActionStore::Action1, 20);

        action_manager.update(80);
        let actions: Vec<_> = action_manager.pull().into();
        match &actions[0] {
            ActionResult::Start { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 0);
                assert_eq!(store.end_time, 10);
                assert!(matches!(store.common(), CommonActionStore::Action1));
            }
            _ => panic!("actions[0] = {:?}", actions[0]),
        };
        match &actions[1] {
            ActionResult::Start { store } => {
                assert_eq!(store.object_id, object2_id);
                assert_eq!(store.start_time, 0);
                assert_eq!(store.end_time, 30);
                assert!(matches!(store.common(), CommonActionStore::Action2));
            }
            _ => panic!("actions[1] = {:?}", actions[1]),
        };
        match &actions[2] {
            ActionResult::End { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 0);
                assert_eq!(store.end_time, 10);
                assert!(matches!(store.common(), CommonActionStore::Action1));
            }
            _ => panic!("actions[2] = {:?}", actions[2]),
        };
        match &actions[3] {
            ActionResult::Start { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 10);
                assert_eq!(store.end_time, 30);
                assert!(matches!(store.common(), CommonActionStore::Action2));
            }
            _ => panic!("actions[3] = {:?}", actions[3]),
        };
        match &actions[4] {
            ActionResult::End { store } => {
                assert_eq!(store.object_id, object2_id);
                assert_eq!(store.start_time, 0);
                assert_eq!(store.end_time, 30);
                assert!(matches!(store.common(), CommonActionStore::Action2));
            }
            _ => panic!("actions[4] = {:?}", actions[4]),
        };
        match &actions[5] {
            ActionResult::End { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 10);
                assert_eq!(store.end_time, 30);
                assert!(matches!(store.common(), CommonActionStore::Action2));
            }
            _ => panic!("actions[5] = {:?}", actions[5]),
        };
        match &actions[6] {
            ActionResult::Start { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 30);
                assert_eq!(store.end_time, 70);
                assert!(matches!(store.common(), CommonActionStore::Action3));
            }
            _ => panic!("actions[6] = {:?}", actions[6]),
        };
        match &actions[7] {
            ActionResult::Start { store } => {
                assert_eq!(store.object_id, object2_id);
                assert_eq!(store.start_time, 30);
                assert_eq!(store.end_time, 40);
                assert!(matches!(store.common(), CommonActionStore::Action3));
            }
            _ => panic!("actions[7] = {:?}", actions[7]),
        };
        match &actions[8] {
            ActionResult::End { store } => {
                assert_eq!(store.object_id, object2_id);
                assert_eq!(store.start_time, 30);
                assert_eq!(store.end_time, 40);
                assert!(matches!(store.common(), CommonActionStore::Action3));
            }
            _ => panic!("actions[8] = {:?}", actions[8]),
        };
        match &actions[9] {
            ActionResult::Start { store } => {
                assert_eq!(store.object_id, object2_id);
                assert_eq!(store.start_time, 40);
                assert_eq!(store.end_time, 100);
                assert!(matches!(store.common(), CommonActionStore::Action4));
            }
            _ => panic!("actions[9] = {:?}", actions[9]),
        };
        match &actions[10] {
            ActionResult::End { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 30);
                assert_eq!(store.end_time, 70);
                assert!(matches!(store.common(), CommonActionStore::Action3));
            }
            _ => panic!("actions[10] = {:?}", actions[10]),
        };
        match &actions[11] {
            ActionResult::Start { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 70);
                assert_eq!(store.end_time, 70);
                assert!(matches!(store.common(), CommonActionStore::Action4));
            }
            _ => panic!("actions[11] = {:?}", actions[11]),
        };
        match &actions[12] {
            ActionResult::End { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 70);
                assert_eq!(store.end_time, 70);
                assert!(matches!(store.common(), CommonActionStore::Action4));
            }
            _ => panic!("actions[12] = {:?}", actions[12]),
        };
        match &actions[13] {
            ActionResult::Update {
                store,
                current_time,
            } => {
                assert_eq!(store.object_id, object2_id);
                assert_eq!(store.start_time, 40);
                assert_eq!(store.end_time, 100);
                assert_eq!(*current_time, 80);
                assert!(matches!(store.common(), CommonActionStore::Action4));
            }
            _ => panic!("actions[13] = {:?}", actions[13]),
        };
        assert_eq!(actions.len(), 14);
    }

    #[test]
    fn test_serialization() {
        let object1_id: GameObjectId = 0;
        let mut action_manager: ActionManager<CommonActionStore> = ActionManager::new();
        action_manager.run(object1_id, CommonActionStore::Action1, 10);
        action_manager.run(object1_id, CommonActionStore::Action2, 20);
        action_manager.run(object1_id, CommonActionStore::Action3, 40);

        action_manager.update(15);
        action_manager.pull();

        let string = serde_json::to_string(action_manager.snapshot()).unwrap();
        let snapshot =
            serde_json::from_str::<ActionManagerSnapshot<CommonActionStore>>(&string).unwrap();
        let mut restored_action_manager = ActionManager::new_with_snapshot(snapshot);

        restored_action_manager.update(60);
        let actions: Vec<_> = restored_action_manager.pull().into();
        match &actions[0] {
            ActionResult::End { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 10);
                assert_eq!(store.end_time, 30);
                assert!(matches!(store.common(), CommonActionStore::Action2));
            }
            _ => panic!("actions[0] = {:?}", actions[0]),
        };
        match &actions[1] {
            ActionResult::Start { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 30);
                assert_eq!(store.end_time, 70);
                assert!(matches!(store.common(), CommonActionStore::Action3));
            }
            _ => panic!("actions[1] = {:?}", actions[1]),
        };
        match &actions[2] {
            ActionResult::End { store } => {
                assert_eq!(store.object_id, object1_id);
                assert_eq!(store.start_time, 30);
                assert_eq!(store.end_time, 70);
                assert!(matches!(store.common(), CommonActionStore::Action3));
            }
            _ => panic!("actions[2] = {:?}", actions[2]),
        };
        assert_eq!(actions.len(), 3);
    }
}
