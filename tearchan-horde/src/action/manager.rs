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
        let mut first_actions: HashMap<EntityId, Arc<Action<T>>> = HashMap::new();
        let current_time = data.current_time;

        for entity_id in data.entity_ids {
            contexts.insert(
                entity_id,
                ActionContext {
                    last_time: current_time,
                    running_end_time: data.current_time,
                    state_len: 0,
                },
            );
        }

        for action in data.actions {
            let entity_id = action.entity_id;
            let context = contexts.entry(entity_id).or_insert_with(|| ActionContext {
                last_time: action.end_time,
                running_end_time: current_time,
                state_len: 0,
            });
            context.state_len += 1;
            if context.last_time < action.end_time {
                context.last_time = action.end_time;
            }

            if current_time >= action.start_time {
                running_actions.push_back(action.end_time, Arc::clone(&action));
                if first_actions
                    .get(&entity_id)
                    .map(|x| x.start_time < action.start_time)
                    .unwrap_or(true)
                {
                    first_actions.insert(entity_id, action);
                }
            } else {
                pending_actions.push_back(action.start_time, action);
            }
        }

        for (entity_id, context) in &mut contexts {
            if context.state_len == 0 {
                pending_cache.insert(*entity_id);
            }
            context.running_end_time = first_actions.get(&entity_id).unwrap().end_time;
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

    pub fn push_states(
        &mut self,
        entity_id: EntityId,
        states: Vec<(T, TimeMilliseconds)>,
    ) -> Vec<Arc<Action<T>>> {
        let mut actions = Vec::new();
        for (state, duration) in states {
            let last_time = self.get_context_mut(entity_id).last_time;
            let end_time = last_time + duration;
            let action = Arc::new(Action::new(entity_id, last_time, end_time, state));
            self.pending_actions
                .push_back(last_time, Arc::clone(&action));
            let context = self.get_context_mut(entity_id);
            context.last_time = end_time;
            context.state_len += 1;

            actions.push(action);
        }
        actions
    }

    pub fn push_actions(&mut self, entity_id: EntityId, mut actions: Vec<Arc<Action<T>>>) {
        actions.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        self.get_context_mut(entity_id).last_time = actions.last().unwrap().end_time;

        for action in actions {
            if action.start_time <= self.current_time {
                if action.start_time == self.current_time
                    && self.get_context_mut(entity_id).state_len == 0
                {
                    self.pending_actions.push_back(action.end_time, action);
                } else {
                    self.running_actions.push_back(action.end_time, action);
                }
            } else {
                self.pending_actions.push_back(action.end_time, action);
            }
            let context = self.get_context_mut(entity_id);
            context.state_len += 1;
        }
    }

    pub fn pull(&mut self) -> DuplicatableBTreeMap<TimeMilliseconds, ActionResult<T>> {
        let mut results = DuplicatableBTreeMap::default();
        let mut results_of_end: HashMap<EntityId, (TimeMilliseconds, ActionResult<T>)> =
            HashMap::new();

        while let Some(action) = self.pending_actions.pop_first_back() {
            if action.start_time() > self.current_time {
                self.pending_actions.push_front(action.start_time(), action);
                break;
            }

            let entity_id = action.entity_id();
            let start_time = action.start_time();
            let end_time = action.end_time();
            let context = self.get_context_mut(entity_id);

            // If there is an action that is earlier than the end time of the action being executed,
            // it will be an interrupt and the end action will be removed.
            let interrupt = start_time < context.running_end_time;
            if interrupt {
                // Add the end because it does not affect the interrupt
                if let Some((end_time, result)) = results_of_end.remove(&entity_id) {
                    if end_time <= start_time {
                        results.push_front(end_time, result);
                    }
                }
            }

            if action.end_time() > self.current_time {
                self.running_actions
                    .push_front(end_time, Arc::clone(&action));
                results.push_back(
                    start_time,
                    ActionResult::Start {
                        action: Arc::clone(&action),
                    },
                );

                // When an action with zero start and end periods is executed on an entity that has no actions,
                // the entity changes to pending state even though it has actions piled up.
                // Therefore, if there is a subsequent action on the entity, the pending state will be released.
                self.pending_cache.remove(&entity_id);

                let context = self.get_context_mut(entity_id);
                context.running_end_time = end_time;
            } else {
                results.push_back(
                    start_time,
                    ActionResult::Start {
                        action: Arc::clone(&action),
                    },
                );

                results_of_end.insert(
                    entity_id,
                    (
                        end_time,
                        ActionResult::End {
                            action: Arc::clone(&action),
                        },
                    ),
                );

                let context = self.get_context_mut(entity_id);
                context.running_end_time = end_time;
                context.state_len -= 1;
                if context.state_len == 0 {
                    self.pending_cache.insert(entity_id);
                }
            }

            if interrupt {
                results.push_back(
                    start_time,
                    ActionResult::Cancel {
                        action: Arc::clone(&action),
                    },
                );
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

        for (_, (end_time, result)) in results_of_end {
            results.push_front(end_time, result);
        }

        let mut visit_map: HashMap<EntityId, (TimeMilliseconds, Arc<Action<T>>)> = HashMap::new();
        for (_, started_stores) in self.running_actions.iter() {
            for action in started_stores.iter() {
                if visit_map
                    .get(&action.entity_id())
                    .map(|(start_time, _)| start_time < &action.start_time)
                    .unwrap_or(true)
                {
                    visit_map.insert(action.entity_id(), (action.start_time, Arc::clone(action)));
                }
            }
        }
        for (_, (_, action)) in visit_map {
            results.push_back(
                self.current_time,
                ActionResult::Update {
                    action,
                    current_time: self.current_time,
                },
            )
        }

        results
    }

    /**
     * Get free entity ids after update_actions
     */
    pub fn clean_pending_entity_ids(&mut self) -> HashSet<EntityId> {
        std::mem::take(&mut self.pending_cache)
    }

    pub fn attach(&mut self, entity_id: EntityId) {
        debug_assert!(
            !self.contexts.contains_key(&entity_id),
            "entity of {} is already attached",
            entity_id
        );

        let context = ActionContext {
            last_time: self.current_time,
            running_end_time: self.current_time,
            state_len: 0,
        };
        self.contexts.insert(entity_id, context);
        self.pending_cache.insert(entity_id);
    }

    pub fn detach(&mut self, entity_id: EntityId) {
        self.contexts.remove(&entity_id);
        self.pending_cache.remove(&entity_id);
    }

    pub fn cancel(&mut self, entity_id: EntityId, immediate: bool) -> Vec<Arc<Action<T>>> {
        let mut canceled_actions = Vec::new();
        let context = self
            .contexts
            .get_mut(&entity_id)
            .expect("invalid entity_id");
        if immediate {
            context.last_time = self.current_time;
            context.state_len = 0;

            self.pending_cache.insert(entity_id);

            for (_, running_actions) in self.running_actions.iter_mut() {
                running_actions.retain(|action| {
                    let is_target = action.entity_id == entity_id;
                    if is_target {
                        canceled_actions.push(Arc::clone(action));
                    }
                    !is_target
                });
            }
        } else {
            // Find a running action of entity_id
            context.last_time = self
                .running_actions
                .iter()
                .find_map(|(_, running_actions)| {
                    running_actions
                        .iter()
                        .find(|action| action.entity_id == entity_id)
                })
                .map(|action| action.end_time)
                .unwrap_or(self.current_time);
            context.state_len = context.state_len.min(1);
        }

        for (_, pending_actions) in self.pending_actions.iter_mut() {
            pending_actions.retain(|action| {
                let is_target = action.entity_id == entity_id;
                if is_target {
                    canceled_actions.push(Arc::clone(action));
                }
                !is_target
            });
        }
        canceled_actions
    }

    pub fn controller(&mut self) -> ActionController<T> {
        ActionController {
            action_manager: self,
        }
    }

    pub fn reader(&self) -> ActionReader<T> {
        ActionReader {
            action_manager: self,
        }
    }

    pub fn current_time(&self) -> TimeMilliseconds {
        self.current_time
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
            running_end_time: self.current_time,
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
    #[inline]
    pub fn attach(&mut self, entity_id: EntityId) {
        self.action_manager.attach(entity_id);
    }

    #[inline]
    pub fn detach(&mut self, entity_id: EntityId) {
        self.action_manager.detach(entity_id);
    }

    #[inline]
    pub fn cancel(&mut self, entity_id: EntityId, immediate: bool) -> Vec<Arc<Action<T>>> {
        self.action_manager.cancel(entity_id, immediate)
    }
}

pub struct ActionReader<'a, T> {
    action_manager: &'a ActionManager<T>,
}

impl<'a, T> ActionReader<'a, T> {
    /// The time for the next action to be processed
    #[inline]
    pub fn current_time(&self) -> TimeMilliseconds {
        self.action_manager.current_time
    }

    /// The time of the entity's last action
    /// When generating a job, it means the start time of the next action
    #[inline]
    pub fn last_time(&self, entity_id: EntityId) -> Option<TimeMilliseconds> {
        self.action_manager
            .contexts
            .get(&entity_id)
            .map(|context| context.last_time)
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

#[cfg(test)]
mod test {
    use crate::action::manager::{ActionManager, ActionManagerData};
    use crate::action::result::ActionResult;
    use crate::action::Action;
    use crate::action::TimeMilliseconds;
    use std::sync::Arc;
    use tearchan_ecs::component::EntityId;
    use tearchan_util::btree::DuplicatableBTreeMap;

    type TestActionState = &'static str;

    fn flatten<T, F>(
        actions: &DuplicatableBTreeMap<TimeMilliseconds, ActionResult<T>>,
        f: F,
    ) -> Vec<Arc<Action<T>>>
    where
        F: FnMut(&ActionResult<T>) -> Option<Arc<Action<T>>> + Copy,
    {
        actions
            .iter()
            .map(|(_, results)| results.iter().filter_map(f).collect::<Vec<_>>())
            .flatten()
            .collect::<Vec<_>>()
    }

    #[test]
    fn test() {
        let mut action_manager: ActionManager<TestActionState> = ActionManager::default();
        action_manager.attach(1);

        action_manager.push_states(
            1,
            vec![
                ("Wake", 1000),
                ("Run", 2000),
                ("Eat", 1000),
                ("Sleep", 3000),
            ],
        );
        assert_eq!(action_manager.contexts.get(&1).unwrap().state_len, 4);
        assert_eq!(action_manager.contexts.get(&1).unwrap().last_time, 7000);

        let mut actions = action_manager.pull();
        let action = actions.pop_first_back().unwrap();
        assert_eq!(action.get_start().unwrap().start_time, 0);
        assert_eq!(action.get_start().unwrap().inner.as_ref(), &"Wake");

        let action = actions.pop_first_back().unwrap();
        assert_eq!(action.get_update().unwrap().0.start_time, 0);
        assert_eq!(action.get_update().unwrap().0.inner.as_ref(), &"Wake");
        assert!(actions.pop_first_back().is_none());

        let mut actions = action_manager.pull();
        let action = actions.pop_first_back().unwrap();
        assert_eq!(action.get_update().unwrap().1, 0);
        assert!(actions.pop_first_back().is_none());

        action_manager.update(1500);

        let mut actions = action_manager.pull();
        let action = actions.pop_first_back().unwrap();
        assert_eq!(action.get_end().unwrap().end_time, 1000);
        assert_eq!(action.get_end().unwrap().inner.as_ref(), &"Wake");

        let action = actions.pop_first_back().unwrap();
        assert_eq!(action.get_start().unwrap().start_time, 1000);
        assert_eq!(action.get_start().unwrap().inner.as_ref(), &"Run");

        let action = actions.pop_first_back().unwrap();
        assert_eq!(action.get_update().unwrap().1, 1500);
        assert_eq!(action.get_update().unwrap().0.inner.as_ref(), &"Run");
        assert!(actions.pop_first_back().is_none());

        let ids = action_manager.clean_pending_entity_ids();
        assert!(!ids.contains(&1));

        action_manager.update(5500);
        action_manager.pull();

        let ids = action_manager.clean_pending_entity_ids();
        assert!(ids.contains(&1));
    }

    #[test]
    fn test_cancel() {
        let mut action_manager: ActionManager<TestActionState> = ActionManager::default();
        action_manager.attach(1);

        action_manager.push_states(
            1,
            vec![
                ("Wake", 1000),
                ("Run", 2000),
                ("Eat", 1000),
                ("Sleep", 3000),
            ],
        );
        action_manager.update(1500);
        action_manager.pull();
        let canceled_actions = action_manager.cancel(1, true);
        assert_eq!(canceled_actions.len(), 3);
        assert_eq!(canceled_actions[0].inner.as_ref(), &"Run");
        assert_eq!(canceled_actions[1].inner.as_ref(), &"Eat");
        assert_eq!(canceled_actions[2].inner.as_ref(), &"Sleep");

        let mut actions = action_manager.pull();
        assert!(actions.pop_first_back().is_none());

        action_manager.push_states(
            1,
            vec![
                ("Wake", 1000),
                ("Run", 2000),
                ("Eat", 1000),
                ("Sleep", 3000),
            ],
        );
        action_manager.update(1500);
        action_manager.pull();
        let canceled_actions = action_manager.cancel(1, false);
        assert_eq!(canceled_actions.len(), 2);
        assert_eq!(canceled_actions[0].inner.as_ref(), &"Eat");
        assert_eq!(canceled_actions[1].inner.as_ref(), &"Sleep");

        let mut actions = action_manager.pull();
        let action = actions.pop_first_back().unwrap();
        assert_eq!(action.get_update().unwrap().1, 3000);
        assert_eq!(action.get_update().unwrap().0.end_time, 4500);
        assert_eq!(action.get_update().unwrap().0.inner.as_ref(), &"Run");

        action_manager.update(2000);
        let mut actions = action_manager.pull();
        let action = actions.pop_first_back().unwrap();
        assert_eq!(action.get_end().unwrap().end_time, 4500);
        assert_eq!(action.get_end().unwrap().inner.as_ref(), &"Run");
        assert!(actions.pop_first_back().is_none());
    }

    #[test]
    fn test_serialization() {
        let mut data: ActionManagerData<TestActionState> = ActionManagerData::default();
        data.current_time = 700;

        data.entity_ids.insert(1);
        data.actions.push(Arc::new(Action::new(1, 0, 500, "Jump")));
        data.actions.push(Arc::new(Action::new(1, 250, 600, "Run")));
        data.actions
            .push(Arc::new(Action::new(1, 600, 800, "Walk")));
        data.actions
            .push(Arc::new(Action::new(1, 800, 1200, "Sleep")));

        let manager = ActionManager::new(data);
        assert_eq!(manager.running_actions.len(), 3);
        assert_eq!(manager.pending_actions.len(), 1);
        assert_eq!(manager.contexts.get(&1).unwrap().last_time, 1200);
        assert_eq!(manager.contexts.get(&1).unwrap().running_end_time, 800);
        assert_eq!(manager.contexts.get(&1).unwrap().state_len, 4);
    }

    #[test]
    fn test_push_actions() {
        let mut action_manager_1: ActionManager<TestActionState> = ActionManager::default();
        let entity_id: EntityId = 1;
        action_manager_1.attach(entity_id);
        action_manager_1.push_states(
            entity_id,
            vec![
                ("Wake", 1000),
                ("Run", 2000),
                ("Eat", 1000),
                ("Sleep", 3000),
            ],
        );
        action_manager_1.update(1500);

        let actions_1 = action_manager_1.pull();
        let actions = flatten(&actions_1, |action| match action {
            ActionResult::Start { action } => Some(Arc::clone(action)),
            ActionResult::Update { .. } => None,
            ActionResult::End { .. } => None,
            ActionResult::Cancel { .. } => None,
        });

        let mut action_manager_2: ActionManager<TestActionState> = ActionManager::default();
        action_manager_2.attach(entity_id);
        action_manager_2.push_actions(entity_id, actions);
        action_manager_2.update(1500);

        let actions_2 = action_manager_2.pull();

        let mut action_1 = actions_1.iter();
        let mut action_2 = actions_2.iter();
        {
            let action_1 = action_1.next().unwrap();
            let action_2 = action_2.next().unwrap();
            assert_eq!(action_1.0, action_2.0);
            assert_eq!(
                action_1.1[0].get_start().unwrap().inner,
                action_2.1[0].get_start().unwrap().inner
            );
        }
        {
            let action_1 = action_1.next().unwrap();
            let action_2 = action_2.next().unwrap();
            assert_eq!(action_1.0, action_2.0);
            assert_eq!(
                action_1.1[0].get_end().unwrap().inner,
                action_2.1[0].get_end().unwrap().inner
            );
            assert_eq!(action_1.0, action_2.0);
            assert_eq!(
                action_1.1[1].get_start().unwrap().inner,
                action_2.1[1].get_start().unwrap().inner
            );
        }
        {
            let action_1 = action_1.next().unwrap();
            let action_2 = action_2.next().unwrap();
            assert_eq!(action_1.0, action_2.0);
            assert_eq!(
                action_1.1[0].get_update().unwrap().0.inner,
                action_2.1[0].get_update().unwrap().0.inner
            );
        }
    }

    #[test]
    fn test_push_actions_with_cancels() {
        let mut action_manager_1: ActionManager<TestActionState> = ActionManager::default();
        let entity_id: EntityId = 1;
        action_manager_1.attach(entity_id);
        action_manager_1.push_states(
            entity_id,
            vec![
                ("Wake", 1000),
                ("Run", 2000),
                ("Eat", 1000),
                ("Sleep", 3000),
            ],
        );
        action_manager_1.update(1500);
        let actions_1 = action_manager_1.pull();
        let mut actions = flatten(&actions_1, |action| match action {
            ActionResult::Start { action } => Some(Arc::clone(action)),
            ActionResult::Update { .. } => None,
            ActionResult::End { .. } => None,
            ActionResult::Cancel { .. } => None,
        });

        println!("1 - {:?}", actions_1);

        action_manager_1.cancel(entity_id, true);
        action_manager_1.push_states(
            entity_id,
            vec![("Drink", 1200), ("Eat", 1000), ("Sleep", 3000)],
        );
        action_manager_1.update(0);
        let actions_1 = action_manager_1.pull();
        actions.append(&mut flatten(&actions_1, |action| match action {
            ActionResult::Start { action } => Some(Arc::clone(action)),
            ActionResult::Update { .. } => None,
            ActionResult::End { .. } => None,
            ActionResult::Cancel { .. } => None,
        }));
        println!("1 - {:?}", actions_1);
        println!("s - {:?}", actions);

        let mut action_manager_2: ActionManager<TestActionState> = ActionManager::default();
        action_manager_2.attach(entity_id);
        action_manager_2.push_actions(entity_id, actions);
        action_manager_2.update(1500);

        let actions_2 = action_manager_2.pull();

        println!("2 - {:?}", actions_2);
    }

    #[test]
    fn test_push_actions_with_cancels2() {
        let entity_id = 1;
        let mut action_manager_2: ActionManager<TestActionState> = ActionManager::default();
        action_manager_2.attach(entity_id);
        action_manager_2.push_actions(
            entity_id,
            vec![
                Arc::new(Action::new(entity_id, 0, 1000, "Wake")),
                Arc::new(Action::new(entity_id, 1000, 3000, "Run")),
                Arc::new(Action::new(entity_id, 1500, 2700, "Drink")),
            ],
        );
        action_manager_2.update(1500);

        let actions_2 = action_manager_2.pull();

        println!("2 - {:?}", actions_2);
    }
}
