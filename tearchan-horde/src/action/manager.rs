use crate::action::context::ActionContext;
use crate::action::result::ActionResult;
use crate::action::Action;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tearchan_ecs::component::EntityId;
use tearchan_util::btree::DuplicatableBTreeMap;

pub type TimeMilliseconds = u64;

pub struct ActionManager<T> {
    // Do serialize
    pending_actions: DuplicatableBTreeMap<TimeMilliseconds, Arc<Action<T>>>,
    running_actions: DuplicatableBTreeMap<TimeMilliseconds, Arc<Action<T>>>,
    current_time: TimeMilliseconds,
    // Cache and using for server only
    contexts: HashMap<EntityId, ActionContext>,
    pending_cache: HashSet<EntityId>,
}

impl<T> Default for ActionManager<T> {
    fn default() -> Self {
        ActionManager {
            pending_actions: DuplicatableBTreeMap::default(),
            running_actions: DuplicatableBTreeMap::default(),
            current_time: 0,
            contexts: HashMap::new(),
            pending_cache: HashSet::new(),
        }
    }
}

impl<T> ActionManager<T> {
    pub fn new(data: ActionManagerData<T>) -> ActionManager<T> {
        let mut pending_actions = DuplicatableBTreeMap::default();
        let mut running_actions = DuplicatableBTreeMap::default();
        let mut contexts = HashMap::new();
        let mut pending_cache = HashSet::new();

        for entity_id in data.entity_ids {
            contexts.insert(
                entity_id,
                ActionContext {
                    last_time: data.current_time,
                    state_len: 0,
                },
            );
        }

        for action in data.actions {
            let context = contexts
                .entry(action.entity_id)
                .or_insert_with(|| ActionContext {
                    last_time: action.end_time,
                    state_len: 0,
                });
            context.state_len += 1;
            if context.last_time < action.end_time {
                context.last_time = action.end_time;
            }

            if data.current_time >= action.start_time {
                running_actions.push_back(action.start_time, action);
            } else {
                pending_actions.push_back(action.start_time, action);
            }
        }

        for (entity_id, context) in &contexts {
            if context.state_len == 0 {
                pending_cache.insert(*entity_id);
            }
        }

        ActionManager {
            pending_actions,
            running_actions,
            current_time: data.current_time,
            contexts,
            pending_cache,
        }
    }

    pub fn update(&mut self, elapsed_time: TimeMilliseconds) {
        self.current_time += elapsed_time;
    }

    pub fn is_running(&self, entity_id: EntityId) -> bool {
        match self.contexts.get(&entity_id) {
            Some(context) => context.state_len != 0,
            None => false,
        }
    }

    pub fn push_states(&mut self, entity_id: EntityId, states: Vec<(T, TimeMilliseconds)>) {
        for (state, duration) in states {
            let last_time = self.get_context_mut(entity_id).last_time;
            let end_time = last_time + duration;
            self.pending_actions.push_back(
                last_time,
                Arc::new(Action::new(entity_id, last_time, end_time, state)),
            );
            self.get_context_mut(entity_id).last_time = end_time;
        }
    }

    pub fn pull(&mut self) -> DuplicatableBTreeMap<u64, ActionResult<T>> {
        let mut results = DuplicatableBTreeMap::default();
        while let Some(action) = self.pending_actions.pop_first_back() {
            if action.start_time() > self.current_time {
                self.pending_actions.push_front(action.start_time(), action);
                break;
            }

            let entity_id = action.entity_id();
            let start_time = action.start_time();
            let end_time = action.end_time();
            if action.end_time() > self.current_time {
                self.running_actions
                    .push_front(end_time, Arc::clone(&action));
                results.push_back(start_time, ActionResult::Start { action });

                let context = self.get_context_mut(entity_id);
                context.state_len += 1;
            } else {
                results.push_back(
                    start_time,
                    ActionResult::Start {
                        action: Arc::clone(&action),
                    },
                );
                results.push_back(end_time, ActionResult::End { action });

                let context = self.get_context_mut(entity_id);
                if context.state_len == 0 {
                    self.pending_cache.insert(entity_id);
                }
            }
        }

        while let Some(action) = self.running_actions.pop_first_back() {
            if action.end_time() > self.current_time {
                self.running_actions.push_front(action.end_time(), action);
                break;
            }

            let entity_id = action.entity_id();
            results.push_front(action.end_time(), ActionResult::End { action });

            let context = self.get_context_mut(entity_id);
            context.state_len -= 1;
            if context.state_len == 0 {
                self.pending_cache.insert(entity_id);
            }
        }

        for (_, started_stores) in self.running_actions.iter() {
            for action in started_stores.iter() {
                results.push_back(
                    self.current_time,
                    ActionResult::Update {
                        action: Arc::clone(action),
                        current_time: self.current_time,
                    },
                )
            }
        }
        results
    }

    /**
     * Get free entity ids after update_actions
     */
    pub fn clean_pending_entity_ids(&mut self) -> HashSet<EntityId> {
        std::mem::replace(&mut self.pending_cache, HashSet::new())
    }

    pub fn attach(&mut self, entity_id: EntityId) {
        debug_assert!(
            !self.contexts.contains_key(&entity_id),
            "entity of {} is already attached",
            entity_id
        );

        let context = ActionContext {
            last_time: self.current_time,
            state_len: 0,
        };
        self.contexts.insert(entity_id, context);
        self.pending_cache.insert(entity_id);
    }

    pub fn detach(&mut self, entity_id: EntityId) {
        self.contexts.remove(&entity_id);
        self.pending_cache.remove(&entity_id);
    }

    pub fn cancel(&mut self, entity_id: EntityId) {
        let context = self
            .contexts
            .get_mut(&entity_id)
            .expect("invalid entity_id");
        context.last_time = self.current_time;
        context.state_len = 0;
        self.pending_cache.insert(entity_id);

        for (_, pending_actions) in self.pending_actions.iter_mut() {
            pending_actions.retain(|action| action.entity_id != entity_id);
        }
        for (_, running_actions) in self.running_actions.iter_mut() {
            running_actions.retain(|action| action.entity_id != entity_id);
        }
    }

    pub fn controller(&mut self) -> ActionController<T> {
        ActionController {
            action_manager: self,
        }
    }

    pub fn create_data(&self) -> ActionManagerData<T> {
        let mut actions = vec![];
        for (_, pending_actions) in self.pending_actions.iter() {
            for pending_action in pending_actions {
                actions.push(Arc::clone(&pending_action));
            }
        }
        for (_, running_actions) in self.running_actions.iter() {
            for running_action in running_actions {
                actions.push(Arc::clone(&running_action));
            }
        }

        ActionManagerData {
            actions,
            entity_ids: self
                .contexts
                .iter()
                .map(|(entity_id, _)| *entity_id)
                .collect(),
            current_time: self.current_time,
        }
    }

    fn get_context_mut(&mut self, entity_id: EntityId) -> &mut ActionContext {
        debug_assert!(
            self.contexts.contains_key(&entity_id),
            "entity of {} is not attached",
            entity_id
        );

        if self.contexts.get_mut(&entity_id).is_some() {
            return self.contexts.get_mut(&entity_id).unwrap();
        }

        let context = ActionContext {
            last_time: self.current_time,
            state_len: 0,
        };
        self.contexts.insert(entity_id, context);
        self.contexts.get_mut(&entity_id).unwrap()
    }
}

pub struct ActionController<'a, T> {
    action_manager: &'a mut ActionManager<T>,
}

impl<'a, T> ActionController<'a, T> {
    pub fn attach(&mut self, entity_id: EntityId) {
        self.action_manager.attach(entity_id);
    }

    pub fn detach(&mut self, entity_id: EntityId) {
        self.action_manager.detach(entity_id);
    }

    pub fn cancel(&mut self, entity_id: EntityId) {
        self.action_manager.cancel(entity_id);
    }
}

#[derive(Serialize, Deserialize)]
pub struct ActionManagerData<T> {
    pub actions: Vec<Arc<Action<T>>>,
    #[serde(rename = "entityIds")]
    pub entity_ids: HashSet<EntityId>,
    #[serde(rename = "currentTime")]
    pub current_time: TimeMilliseconds,
}

impl<T> Default for ActionManagerData<T> {
    fn default() -> Self {
        ActionManagerData {
            actions: Vec::new(),
            entity_ids: HashSet::new(),
            current_time: 0,
        }
    }
}
