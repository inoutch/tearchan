use crate::action::context::ActionContext;
use crate::action::result::ActionResult;
use crate::action::Action;
use crate::HordeInterface;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use tearchan_ecs::component::EntityId;
use tearchan_util::btree::DuplicatableBTreeMap;

pub type TimeMilliseconds = u64;

pub struct ActionManager<T>
where
    T: HordeInterface,
{
    // Do serialize
    pending_actions: DuplicatableBTreeMap<TimeMilliseconds, Rc<Action<T::ActionState>>>,
    running_actions: DuplicatableBTreeMap<TimeMilliseconds, Rc<Action<T::ActionState>>>,
    current_time: TimeMilliseconds,
    // Cache and using for server only
    contexts: HashMap<EntityId, ActionContext>,
    pending_cache: HashSet<EntityId>,
}

impl<T> Default for ActionManager<T>
where
    T: HordeInterface,
{
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

impl<T> ActionManager<T>
where
    T: HordeInterface,
{
    pub fn update(&mut self, elapsed_time: TimeMilliseconds) {
        self.current_time += elapsed_time;
    }

    pub fn is_running(&self, entity_id: EntityId) -> bool {
        match self.contexts.get(&entity_id) {
            Some(context) => context.state_len != 0,
            None => false,
        }
    }

    pub fn push_states(
        &mut self,
        entity_id: EntityId,
        states: Vec<(T::ActionState, TimeMilliseconds)>,
    ) {
        for (state, duration) in states {
            let last_time = self.get_context_mut(entity_id).last_time;
            let end_time = last_time + duration;
            self.pending_actions.push_back(
                last_time,
                Rc::new(Action::new(entity_id, last_time, end_time, state)),
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
                    .push_front(end_time, Rc::clone(&action));
                results.push_back(start_time, ActionResult::Start { action });

                let context = self.get_context_mut(entity_id);
                context.state_len += 1;
            } else {
                results.push_back(
                    start_time,
                    ActionResult::Start {
                        action: Rc::clone(&action),
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
                        action: Rc::clone(action),
                        current_time: self.current_time,
                    },
                )
            }
        }
        results
    }

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
