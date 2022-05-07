use crate::action::manager::TimeMilliseconds;
use crate::v2::action::collection::{
    ActionMeta, TypedAnyActionMap, TypedAnyActionMapGroupedByEntityId,
};
use crate::v2::action::{Action, ActionSessionId, ActionType, ArcAction, VALID_SESSION_ID};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use tearchan_ecs::component::EntityId;
use tearchan_ecs::entity::manager::ENTITY_REMAPPER;
use tearchan_util::id_manager::IdManager;

#[derive(Default)]
pub struct ActionSessionValidator<'a> {
    contexts: Option<&'a HashMap<EntityId, ActionContext>>,
}

impl<'a> ActionSessionValidator<'a> {
    pub fn validate(&self, action: &ActionMeta) -> bool {
        let contexts = match self.contexts {
            None => return true,
            Some(contexts) => contexts,
        };
        contexts
            .get(&action.entity_id)
            .map(|context| {
                action.session_id == context.session_id
                    || action.session_id == VALID_SESSION_ID
                    || action.tick <= context.session_expired_at
            })
            .unwrap_or(false)
    }
}
// x1/4 -> 60 / 4 = 15
// 1 tick is 66ms
const TICK_DURATION: u64 = 1000 / 15;
type Tick = u64;

enum Event {
    Started {
        type_id: TypeId,
        update_action: Box<dyn Any>,
        running_end_tick: Tick,
    },
    Ended,
    Canceled,
}

struct ActionContext {
    last_tick: Tick,             // The last time of stacking all actions
    running_end_tick: Tick,      // The end time of running action
    session_id: ActionSessionId, // Use for validation action
    session_expired_at: Tick,    // Invalid session is expired at this
}

#[derive(Default)]
struct BundleForeachTick {
    map: TypedAnyActionMap,
    events: VecDeque<(EntityId, (ActionSessionId, Event))>,
    cancels: BTreeSet<EntityId>,
}

pub struct PullActionResult {
    pub map: TypedAnyActionMap,
    pub cancels: BTreeSet<EntityId>,
}

pub struct ActionManager {
    tick_duration: TimeMilliseconds,
    next_time: TimeMilliseconds,
    next_tick: Tick,
    current_tick: Tick,
    actions: BTreeMap<Tick, BundleForeachTick>,
    update_actions: TypedAnyActionMapGroupedByEntityId,
    contexts: HashMap<EntityId, ActionContext>,
    vacated_entities: BTreeSet<EntityId>,
    session_id_manager: IdManager<ActionSessionId>,
}

impl Default for ActionManager {
    fn default() -> Self {
        ActionManager {
            tick_duration: TICK_DURATION,
            next_time: 0,
            next_tick: 0,
            current_tick: 0,
            actions: Default::default(),
            update_actions: Default::default(),
            contexts: Default::default(),
            vacated_entities: Default::default(),
            session_id_manager: IdManager::new(ActionSessionId::default(), |id| id.next()),
        }
    }
}

impl ActionManager {
    pub fn attach(&mut self, entity_id: EntityId) {
        self.controller().attach(entity_id);
    }

    pub fn detach(&mut self, entity_id: EntityId) {
        self.controller().detach(entity_id);
    }

    pub fn update(&mut self, delta: TimeMilliseconds) {
        debug_assert!(self.vacated_entities.is_empty());

        self.next_time += delta;
        self.next_tick = self.next_time / self.tick_duration;
    }

    pub fn enqueue<T>(&mut self, entity_id: EntityId, raw: Arc<T>, duration: TimeMilliseconds)
    where
        T: 'static,
    {
        self.controller().enqueue(entity_id, raw, duration);
    }

    pub fn interrupt<T>(&mut self, entity_id: EntityId, raw: Arc<T>, duration: TimeMilliseconds)
    where
        T: 'static,
    {
        self.cancel(entity_id, true);
        self.enqueue(entity_id, raw, duration);
    }

    pub fn pull_actions(&mut self) -> Option<PullActionResult> {
        let (tick, mut item) = self.actions.pop_first()?;
        if tick > self.next_tick {
            self.current_tick = self.next_tick;
            self.actions.insert(tick, item);
            return None;
        }

        self.current_tick = tick;

        while let Some((entity_id, (session_id, event))) = item.events.pop_front() {
            let context = match self.contexts.get_mut(&entity_id) {
                Some(context) => context,
                None => continue,
            };
            if context.session_id != session_id {
                continue;
            }
            match event {
                Event::Started {
                    type_id,
                    update_action: action,
                    running_end_tick,
                } => {
                    self.update_actions
                        .insert_with_type_id(entity_id, action, type_id);
                    context.running_end_tick = running_end_tick;
                }
                Event::Ended => {
                    debug_assert!(self.current_tick <= context.last_tick);
                    if context.last_tick == self.current_tick {
                        self.vacated_entities.insert(entity_id);
                    }
                }
                Event::Canceled => {
                    self.vacated_entities.insert(entity_id);
                }
            }
        }

        Some(PullActionResult {
            map: item.map,
            cancels: item.cancels,
        })
    }

    pub fn pull_updates(&self) -> &TypedAnyActionMapGroupedByEntityId {
        &self.update_actions
    }

    pub fn pull_vacated_entities(&mut self) -> BTreeSet<EntityId> {
        std::mem::take(&mut self.vacated_entities)
    }

    pub fn get_vacated_entities(&self) -> &BTreeSet<EntityId> {
        &self.vacated_entities
    }

    pub fn cancel(&mut self, entity_id: EntityId, immediate: bool) {
        self.controller().cancel(entity_id, immediate);
    }

    pub fn validator(&self) -> ActionSessionValidator {
        ActionSessionValidator {
            contexts: Some(&self.contexts),
        }
    }

    #[inline]
    pub fn current_tick(&self) -> Tick {
        self.current_tick
    }

    #[inline]
    pub fn next_time(&self) -> TimeMilliseconds {
        self.next_time
    }

    pub fn to_data<T>(
        &self,
        converter0: fn(&TypedAnyActionMap, &ActionSessionValidator) -> Vec<Action<T>>,
        converter1: fn(&TypedAnyActionMapGroupedByEntityId) -> Vec<Action<T>>,
    ) -> ActionManagerData<T> {
        debug_assert!(
            self.vacated_entities.is_empty(),
            "Must process vacated entities until busy"
        );
        if let Some((tick, _)) = self.actions.iter().next() {
            debug_assert!(self.current_tick < *tick, "Must process current actions");
        }

        let validator = ActionSessionValidator {
            contexts: Some(&self.contexts),
        };
        let mut actions = converter1(&self.update_actions);
        actions.append(
            &mut self
                .actions
                .iter()
                .flat_map(|(_tick, action)| converter0(&action.map, &validator))
                .collect::<Vec<_>>(),
        );
        ActionManagerData {
            actions,
            tick_duration: self.tick_duration,
            current_tick: self.current_tick,
            next_time: self.next_time,
        }
    }

    pub fn from_data<T>(
        data: ActionManagerData<T>,
        converter: fn(action: Action<T>, manager: &mut ActionManagerConverter),
    ) -> Result<Self, ActionManagerError> {
        let mut manager = ActionManager {
            next_time: data.next_time,
            next_tick: data.current_tick,
            current_tick: data.current_tick,
            ..Default::default()
        };
        let mut tick_validation: Tick = data.current_tick;
        for action in data.actions.iter() {
            if let std::collections::hash_map::Entry::Vacant(entry) =
                manager.contexts.entry(action.entity_id)
            {
                entry.insert(ActionContext {
                    last_tick: manager.current_tick,
                    running_end_tick: Tick::MAX,
                    session_id: manager.session_id_manager.gen(),
                    session_expired_at: manager.current_tick,
                });
            }
            match action.ty {
                ActionType::Start { .. } | ActionType::End { .. } => {
                    let tick = action.tick().unwrap();
                    if tick < tick_validation {
                        return Err(ActionManagerError::InvalidDataCauseBySortedActions);
                    }
                    tick_validation = tick;
                }
                ActionType::Update { .. } => {
                    if tick_validation != data.current_tick {
                        return Err(ActionManagerError::InvalidDataCauseBySortedActions);
                    }
                }
            }
        }

        let mut c = ActionManagerConverter {
            remapping_tick: 0,
            remapping_tick_positive: true,
            tick_duration: data.tick_duration,
            actions: &mut manager.actions,
            contexts: &mut manager.contexts,
            update_actions: &mut manager.update_actions,
        };
        for action in data.actions.into_iter() {
            converter(action, &mut c);
        }

        for (_entity, context) in manager.contexts.iter() {
            if context.running_end_tick == Tick::MAX {
                return Err(ActionManagerError::InvalidDataNoEndAction);
            }
        }

        Ok(manager)
    }

    pub fn load_data<T>(
        &mut self,
        data: ActionManagerData<T>,
        converter: fn(action: Action<T>, manager: &mut ActionManagerConverter),
    ) -> Result<ActionRemapperToken, ActionManagerError> {
        let mut tick_validation: Tick = data.current_tick;
        for action in data.actions.iter() {
            match action.ty {
                ActionType::Start { .. } | ActionType::End { .. } => {
                    let tick = action.tick().unwrap();
                    if tick < tick_validation {
                        return Err(ActionManagerError::InvalidDataCauseBySortedActions);
                    }
                    tick_validation = tick;
                }
                ActionType::Update { .. } => {
                    if tick_validation != data.current_tick {
                        return Err(ActionManagerError::InvalidDataCauseBySortedActions);
                    }
                }
            }
        }
        for action in data.actions.iter() {
            if let std::collections::hash_map::Entry::Vacant(entry) =
                self.contexts.entry(ENTITY_REMAPPER.remap(action.entity_id))
            {
                entry.insert(ActionContext {
                    last_tick: self.current_tick,
                    running_end_tick: Tick::MAX,
                    session_id: self.session_id_manager.gen(),
                    session_expired_at: self.current_tick,
                });
            }
        }

        let remapping_tick = (self.current_tick as i128 - data.current_tick as i128).abs() as Tick;
        let remapping_tick_positive = self.current_tick > data.current_tick;
        let mut c = ActionManagerConverter {
            remapping_tick,
            remapping_tick_positive,
            tick_duration: data.tick_duration,
            actions: &mut self.actions,
            contexts: &mut self.contexts,
            update_actions: &mut self.update_actions,
        };
        for action in data.actions.into_iter() {
            converter(action, &mut c);
        }

        for (_entity, context) in self.contexts.iter() {
            if context.running_end_tick == Tick::MAX {
                return Err(ActionManagerError::InvalidDataNoEndAction);
            }
        }

        Ok(ActionRemapperToken::new(
            remapping_tick,
            remapping_tick_positive,
        ))
    }

    pub fn controller(&mut self) -> ActionController {
        ActionController {
            tick_duration: self.tick_duration,
            current_tick: self.current_tick,
            actions: &mut self.actions,
            update_actions: &mut self.update_actions,
            contexts: &mut self.contexts,
            vacated_entities: &mut self.vacated_entities,
            session_id_manager: &mut self.session_id_manager,
        }
    }

    pub fn has_some_actions(&self, entity_id: EntityId) -> bool {
        let context = match self.contexts.get(&entity_id) {
            None => return false,
            Some(context) => context,
        };
        context.last_tick != self.current_tick
    }
}

pub struct ActionManagerConverter<'a> {
    remapping_tick: Tick,
    remapping_tick_positive: bool,
    tick_duration: TimeMilliseconds,
    actions: &'a mut BTreeMap<Tick, BundleForeachTick>,
    contexts: &'a mut HashMap<EntityId, ActionContext>,
    update_actions: &'a mut TypedAnyActionMapGroupedByEntityId,
}

impl<'a> ActionManagerConverter<'a> {
    pub fn load<T>(&mut self, action: ArcAction<T>)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let entity_id = ENTITY_REMAPPER.remap(action.entity_id);
        let context = self
            .contexts
            .get_mut(&entity_id)
            .unwrap_or_else(|| panic!("entity of {} is not attached", entity_id));

        let remapped_action_type = action.ty.remap(
            self.remapping_tick,
            self.remapping_tick_positive,
            self.tick_duration,
        );
        match remapped_action_type {
            ActionType::Start { start, end } => {
                let tick = remapped_action_type.tick().unwrap();
                let item = self.actions.entry(tick).or_insert_with(Default::default);
                let start_time = start.wrapping_mul(self.tick_duration);
                let end_time = end.wrapping_mul(self.tick_duration);
                item.events.push_back((
                    entity_id,
                    (
                        context.session_id,
                        Event::Started {
                            type_id,
                            update_action: Box::new(Action {
                                raw: Arc::clone(&action.raw),
                                entity_id,
                                ty: ActionType::Update {
                                    start: start_time,
                                    end: end_time,
                                },
                            }),
                            running_end_tick: end,
                        },
                    ),
                ));
                item.map.push(
                    Action {
                        raw: Arc::clone(action.raw()),
                        entity_id,
                        ty: remapped_action_type,
                    },
                    context.session_id,
                );
            }
            ActionType::End { .. } => {
                let tick = remapped_action_type.tick().unwrap();
                let item = self.actions.entry(tick).or_insert_with(Default::default);
                item.events
                    .push_back((entity_id, (context.session_id, Event::Ended)));
                item.map.push(
                    Action {
                        raw: Arc::clone(action.raw()),
                        entity_id,
                        ty: remapped_action_type,
                    },
                    context.session_id,
                );
                context.last_tick = context.last_tick.max(tick);
                context.running_end_tick = context.running_end_tick.min(tick);
            }
            ActionType::Update { .. } => {
                self.update_actions.insert(
                    entity_id,
                    Action {
                        raw: Arc::clone(action.raw()),
                        entity_id,
                        ty: remapped_action_type,
                    },
                );
            }
        }
    }
}

pub struct ActionController<'a> {
    tick_duration: TimeMilliseconds,
    current_tick: Tick,
    actions: &'a mut BTreeMap<Tick, BundleForeachTick>,
    update_actions: &'a mut TypedAnyActionMapGroupedByEntityId,
    contexts: &'a mut HashMap<EntityId, ActionContext>,
    vacated_entities: &'a mut BTreeSet<EntityId>,
    session_id_manager: &'a mut IdManager<ActionSessionId>,
}

impl<'a> ActionController<'a> {
    #[inline]
    pub fn enqueue<T>(&mut self, entity_id: EntityId, raw: Arc<T>, duration: TimeMilliseconds)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let context = self
            .contexts
            .get_mut(&entity_id)
            .unwrap_or_else(|| panic!("entity of {} is not attached", entity_id));

        debug_assert!(
            self.current_tick <= context.running_end_tick,
            "{} <= {}",
            self.current_tick,
            context.running_end_tick
        );
        debug_assert!(
            context.running_end_tick <= context.last_tick,
            "{} <= {}",
            context.running_end_tick,
            context.last_tick
        );

        self.vacated_entities.remove(&entity_id);

        let start_tick = context.last_tick;
        let end_tick = start_tick + duration / self.tick_duration;
        let start = start_tick * self.tick_duration;
        let end = end_tick.wrapping_mul(self.tick_duration);
        {
            let item = self
                .actions
                .entry(start_tick)
                .or_insert_with(Default::default);
            item.map.push(
                Action {
                    raw: Arc::clone(&raw),
                    entity_id,
                    ty: ActionType::Start {
                        start: start_tick,
                        end: end_tick,
                    },
                },
                context.session_id,
            );
            item.events.push_back((
                entity_id,
                (
                    context.session_id,
                    Event::Started {
                        type_id,
                        update_action: Box::new(Action {
                            raw: Arc::clone(&raw),
                            entity_id,
                            ty: ActionType::Update { start, end },
                        }),
                        running_end_tick: end_tick,
                    },
                ),
            ));
        }
        {
            let item = self
                .actions
                .entry(end_tick)
                .or_insert_with(Default::default);
            item.map.push(
                Action {
                    raw,
                    entity_id,
                    ty: ActionType::End {
                        start: start_tick,
                        end: end_tick,
                    },
                },
                context.session_id,
            );
            item.events
                .push_back((entity_id, (context.session_id, Event::Ended)));
        }
        context.last_tick = end_tick;
    }

    #[inline]
    pub fn current_tick(&self) -> Tick {
        self.current_tick
    }

    #[inline]
    pub fn attach(&mut self, entity_id: EntityId) {
        debug_assert!(!self.contexts.contains_key(&entity_id));

        self.contexts.insert(
            entity_id,
            ActionContext {
                last_tick: self.current_tick,
                running_end_tick: self.current_tick,
                session_id: self.session_id_manager.gen(),
                session_expired_at: self.current_tick,
            },
        );
        self.vacated_entities.insert(entity_id);
    }

    #[inline]
    pub fn detach(&mut self, entity_id: EntityId) {
        self.contexts.remove(&entity_id);
        self.vacated_entities.remove(&entity_id);
        self.update_actions.remove(entity_id);
    }

    #[inline]
    pub fn cancel(&mut self, entity_id: EntityId, immediate: bool) {
        self.update_actions.remove(entity_id);

        let context = self.contexts.get_mut(&entity_id).unwrap();
        context.session_id = self.session_id_manager.gen();
        let tick = if immediate {
            self.current_tick
        } else {
            context.running_end_tick
        };
        context.last_tick = tick;
        context.running_end_tick = tick;
        context.session_expired_at = tick;

        let item = self.actions.entry(tick).or_insert_with(Default::default);
        item.cancels.insert(entity_id);
        item.events
            .push_back((entity_id, (context.session_id, Event::Canceled)));
    }
}

#[derive(Debug)]
pub enum ActionManagerError {
    InvalidDataCauseBySortedActions,
    InvalidDataNoEndAction,
}

#[derive(Serialize, Deserialize)]
pub struct ActionManagerData<T> {
    actions: Vec<Action<T>>,
    tick_duration: TimeMilliseconds,
    current_tick: Tick,
    next_time: TimeMilliseconds,
}

#[derive(Default)]
pub struct ActionRemapper {
    mapping: Mutex<Option<(Tick, bool)>>,
}

impl ActionRemapper {
    pub fn remap(&self, tick: Tick) -> Tick {
        self.mapping
            .lock()
            .unwrap()
            .map(|(value, positive)| {
                if positive {
                    tick.wrapping_add(value)
                } else {
                    tick.wrapping_sub(value)
                }
            })
            .unwrap_or(tick)
    }
}

pub static ACTION_REMAPPER: Lazy<ActionRemapper> = Lazy::new(ActionRemapper::default);
static ACTION_REMAPPER_WRITE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub struct ActionRemapperToken<'a> {
    _guard: MutexGuard<'a, ()>,
}

impl<'a> ActionRemapperToken<'a> {
    fn new(tick: Tick, positive: bool) -> Self {
        let guard = ACTION_REMAPPER_WRITE_LOCK.lock().unwrap();
        *ACTION_REMAPPER.mapping.lock().unwrap() = Some((tick, positive));
        ActionRemapperToken { _guard: guard }
    }
}

impl<'a> Drop for ActionRemapperToken<'a> {
    fn drop(&mut self) {
        *ACTION_REMAPPER.mapping.lock().unwrap() = None;
    }
}

#[cfg(test)]
#[macro_use]
mod test {
    use crate::define_actions;
    use crate::v2::action::collection::TypedAnyActionMap;
    use crate::v2::action::manager::{
        ActionManager, ActionManagerData, ActionSessionValidator, PullActionResult, Tick,
    };
    use crate::v2::action::ArcAction;
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeSet;
    use std::fmt::Debug;
    use std::sync::Arc;
    use tearchan_ecs::component::EntityId;

    #[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
    pub struct MoveState;

    #[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
    pub struct JumpState;

    #[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
    pub struct TalkState;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct SnapshotResult<'a> {
        tag: &'static str,
        current_tick: Tick,
        move_actions: Option<Vec<&'a ArcAction<MoveState>>>,
        jump_actions: Option<Vec<&'a ArcAction<JumpState>>>,
        talk_actions: Option<Vec<&'a ArcAction<TalkState>>>,
    }

    impl<'a> SnapshotResult<'a> {
        pub fn from_result(
            tag: &'static str,
            current_tick: Tick,
            result: &'a PullActionResult,
            validator: &ActionSessionValidator,
        ) -> Self {
            SnapshotResult {
                tag,
                current_tick,
                move_actions: result.map.get(validator),
                jump_actions: result.map.get(validator),
                talk_actions: result.map.get(validator),
            }
        }
    }

    define_actions!(
        TestAction,
        (Move, MoveState),
        (Jump, JumpState),
        (Talk, TalkState)
    );

    #[allow(dead_code)]
    fn debug_pull_action_result(
        tag: &str,
        result: &Option<PullActionResult>,
        validator: &ActionSessionValidator,
    ) {
        if let Some(result) = result {
            println!(
                "[{}] move_states: {:?}",
                tag,
                result.map.get::<MoveState>(validator)
            );
            println!(
                "[{}] jump_states: {:?}",
                tag,
                result.map.get::<JumpState>(validator)
            );
            println!(
                "[{}] talk_states: {:?}",
                tag,
                result.map.get::<TalkState>(validator)
            );
        }
    }

    fn assert_snapshot(manager: &mut ActionManager) {
        while let Some(actions) = manager.pull_actions() {
            insta::assert_debug_snapshot!(SnapshotResult {
                current_tick: manager.current_tick(),
                tag: "changes",
                move_actions: actions.map.get(&manager.validator()),
                jump_actions: actions.map.get(&manager.validator()),
                talk_actions: actions.map.get(&manager.validator())
            });
        }
        assert_snapshot_update(manager);
    }

    fn assert_snapshot_update(manager: &mut ActionManager) {
        let actions = manager.pull_updates();

        insta::assert_debug_snapshot!(SnapshotResult {
            current_tick: manager.current_tick(),
            tag: "updates",
            move_actions: actions.get(),
            jump_actions: actions.get(),
            talk_actions: actions.get()
        });
    }

    fn assert_actions<T>(actions0: Option<Vec<&ArcAction<T>>>, actions1: Option<Vec<&ArcAction<T>>>)
    where
        T: 'static + Eq + Debug,
    {
        assert_eq!(actions0.is_none(), actions1.is_none());
        if let Some(actions0) = actions0 {
            let actions1 = actions1.unwrap();
            assert_eq!(actions0.len(), actions1.len());
            for (action0, action1) in actions0.iter().zip(actions1.iter()) {
                assert_eq!(action0.entity_id, action1.entity_id);
                assert_eq!(action0.ty, action1.ty);
                assert_eq!(action0.raw, action1.raw);
            }
        }
    }

    fn assert_action_map<T>(
        map0: &TypedAnyActionMap,
        map1: &TypedAnyActionMap,
        manager0: &ActionManager,
        manager1: &ActionManager,
    ) where
        T: 'static + Eq + Debug,
    {
        let actions0 = map0.get::<T>(&manager0.validator());
        let actions1 = map1.get::<T>(&manager1.validator());
        assert_actions(actions0, actions1);
    }

    fn assert_managers(
        manager0: &mut ActionManager,
        manager1: &mut ActionManager,
    ) -> (usize, Option<BTreeSet<EntityId>>) {
        let mut tick_count = 0;
        loop {
            let result0 = manager0.pull_actions();
            let result1 = manager1.pull_actions();
            if result0.is_none() != result1.is_none() {
                debug_pull_action_result("result0", &result0, &manager0.validator());
                debug_pull_action_result("result1", &result1, &manager0.validator());
                assert_eq!(result0.is_none(), result1.is_none());
            }

            if let Some(result0) = result0 {
                let result1 = result1.unwrap();
                assert_action_map::<MoveState>(&result0.map, &result1.map, manager0, manager1);
                assert_action_map::<JumpState>(&result0.map, &result1.map, manager0, manager1);
                assert_action_map::<TalkState>(&result0.map, &result1.map, manager0, manager1);
            } else {
                break;
            }
            tick_count += 1;

            let vacated_entities0 = manager0.pull_vacated_entities();
            let vacated_entities1 = manager1.pull_vacated_entities();

            assert_eq!(vacated_entities0, vacated_entities1);

            if !vacated_entities0.is_empty() {
                return (tick_count, Some(vacated_entities0));
            }
        }

        let map0 = manager0.pull_updates();
        let map1 = manager1.pull_updates();

        assert_actions::<MoveState>(map0.get(), map1.get());
        assert_actions::<JumpState>(map0.get(), map1.get());
        assert_actions::<TalkState>(map0.get(), map1.get());

        (tick_count, None)
    }

    #[test]
    fn test() {
        // --- Millis: 0, Tick: 0 ---- //
        let mut manager = ActionManager::default();
        manager.attach(1);
        manager.attach(2);

        manager.enqueue(1, Arc::new(MoveState), 1000); // tick: 15
        manager.enqueue(1, Arc::new(JumpState), 2000); // tick: 30(45)
        manager.enqueue(1, Arc::new(TalkState), 1500); // tick: 22(67)

        manager.enqueue(2, Arc::new(MoveState), 500); // tick: 7
        manager.enqueue(2, Arc::new(JumpState), 1500); // tick: 22(29)
        manager.enqueue(2, Arc::new(TalkState), 2000); // tick: 30(59)

        assert_snapshot(&mut manager);

        // --- Tick: 3 ---- //
        manager.update(200);

        assert_snapshot(&mut manager);

        // --- Tick: 7 ---- //
        manager.update(300);

        assert_snapshot(&mut manager);

        // --- Tick: 60
        manager.update(3328);

        assert_snapshot(&mut manager);
    }

    #[test]
    fn test_vacated_entities() {
        let mut manager = ActionManager::default();
        manager.attach(1);
        manager.attach(2);

        manager.enqueue(1, Arc::new(MoveState), 1000); // tick: 15
        manager.enqueue(2, Arc::new(MoveState), 500); // tick: 7

        assert!(manager.pull_vacated_entities().is_empty());
        assert!(manager.has_some_actions(1));
        assert!(manager.has_some_actions(2));

        // Just ended time
        manager.update(500);

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        assert!(manager.pull_actions().is_none());
        assert_eq!(
            manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![2]
        );
        assert!(!manager.has_some_actions(2));

        manager.enqueue(2, Arc::new(JumpState), 2000);

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        assert!(manager.pull_actions().is_none());
        assert!(manager.pull_vacated_entities().is_empty());

        manager.update(10000);

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        assert_eq!(
            manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert!(!manager.has_some_actions(1));
        assert!(manager.has_some_actions(2));

        manager.enqueue(1, Arc::new(TalkState), 3000);

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        assert!(manager.pull_vacated_entities().is_empty());

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        assert_eq!(
            manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![2]
        );
        assert!(manager.has_some_actions(1));
        assert!(!manager.has_some_actions(2));
    }

    #[test]
    fn test_cancel() {
        let mut manager = ActionManager::default();
        manager.attach(1);
        manager.attach(2);

        manager.enqueue(1, Arc::new(MoveState), 1000); // tick: 15
        manager.enqueue(1, Arc::new(JumpState), 2000); // tick: 30(45)
        manager.enqueue(1, Arc::new(TalkState), 1500); // tick: 22(67)

        manager.enqueue(2, Arc::new(MoveState), 500); // tick: 7
        manager.enqueue(2, Arc::new(JumpState), 1500); // tick: 22(29)
        manager.enqueue(2, Arc::new(TalkState), 2000); // tick: 30(59)
        manager.enqueue(2, Arc::new(MoveState), 3000); // tick: 45(104)

        manager.update(2112); // tick: 32

        let result = manager.pull_actions(); // tick: 0 - start1 start2
        assert_eq!(manager.current_tick(), 0);
        assert_eq!(result.unwrap().cancels.len(), 0);
        let result = manager.pull_actions(); // tick: 7 - end2 start2
        assert_eq!(result.unwrap().cancels.len(), 0);
        let result = manager.pull_actions(); // tick: 15 - end1 end2
        assert_eq!(result.unwrap().cancels.len(), 0);
        let result = manager.pull_actions(); // tick: 29 - end2 start2
        assert_eq!(result.unwrap().cancels.len(), 0);

        assert_eq!(manager.current_tick(), 29);

        manager.cancel(1, true);

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));
        assert_eq!(result.cancels.into_iter().collect::<Vec<_>>(), vec![1]);
        assert_eq!(
            manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![1]
        );

        manager.enqueue(1, Arc::new(TalkState), 2000); // tick: 30(59)

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));
        assert_eq!(result.cancels.len(), 0);

        assert!(manager.pull_actions().is_none());

        manager.update(396); // tick: 38

        assert!(manager.pull_actions().is_none());
        assert_eq!(manager.current_tick(), 38);

        manager.cancel(2, false);

        assert!(manager.pull_actions().is_none());

        manager.update(132); // tick: 40

        assert!(manager.pull_actions().is_none());
        assert_eq!(manager.current_tick(), 40);

        manager.update(1254); // tick: 59

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));
        assert_eq!(result.cancels.len(), 0);
        assert_eq!(manager.pull_vacated_entities().len(), 0);

        let result = manager.pull_actions().unwrap();
        assert_eq!(manager.current_tick(), 59);
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));
        assert_eq!(result.cancels.into_iter().collect::<Vec<_>>(), vec![2]);
        assert_eq!(
            manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn test_zero_time_action() {
        let mut manager = ActionManager::default();
        manager.attach(1);
        manager.attach(2);

        manager.enqueue(1, Arc::new(MoveState), 1000); // tick: 15
        manager.enqueue(2, Arc::new(MoveState), 500); // tick: 7

        manager.update(462);

        manager.pull_actions().unwrap();
        manager.pull_actions().unwrap();

        assert_eq!(
            manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![2]
        );

        manager.enqueue(2, Arc::new(JumpState), 0);
        let result = manager.pull_actions().unwrap();
        assert_eq!(manager.current_tick(), 7);
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        assert_eq!(
            manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![2]
        );

        manager.enqueue(2, Arc::new(TalkState), 1000);

        let result = manager.pull_actions().unwrap();
        assert_eq!(manager.current_tick(), 7);
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        manager.update(0);

        manager.cancel(2, true);

        let result = manager.pull_actions().unwrap();
        assert_eq!(manager.current_tick(), 7);
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        assert_eq!(result.cancels.into_iter().collect::<Vec<_>>(), vec![2]);
        assert_eq!(
            manager
                .pull_vacated_entities()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![2]
        );
    }

    #[test]
    fn test_detach() {
        let mut manager = ActionManager::default();
        manager.attach(1);
        manager.attach(2);

        manager.enqueue(1, Arc::new(MoveState), 1000); // tick: 15
        manager.enqueue(2, Arc::new(MoveState), 500); // tick: 7

        manager.update(396);

        manager.pull_actions().unwrap();

        manager.detach(1);
        manager.detach(2);

        assert!(manager.contexts.get(&1).is_none());
        assert!(manager.contexts.get(&2).is_none());
        assert_eq!(manager.pull_vacated_entities().len(), 0);
        assert!(manager.pull_updates().get::<MoveState>().is_none());

        assert!(manager.pull_actions().is_none());
        assert_eq!(manager.pull_vacated_entities().len(), 0);

        manager.update(1000);

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        let result = manager.pull_actions().unwrap();
        insta::assert_debug_snapshot!(SnapshotResult::from_result(
            "changes",
            manager.current_tick(),
            &result,
            &manager.validator()
        ));

        assert!(manager.pull_actions().is_none());
    }

    #[test]
    fn test_serialization() {
        let mut manager = ActionManager::default();
        manager.attach(1);
        manager.attach(2);

        manager.enqueue(1, Arc::new(MoveState), 3000);
        manager.enqueue(1, Arc::new(JumpState), 500);
        manager.enqueue(1, Arc::new(TalkState), 1500);
        manager.enqueue(1, Arc::new(JumpState), 2000);
        manager.enqueue(1, Arc::new(MoveState), 3000);
        manager.enqueue(1, Arc::new(TalkState), 1000);

        manager.enqueue(2, Arc::new(MoveState), 500);
        manager.enqueue(2, Arc::new(TalkState), 1000);
        manager.enqueue(2, Arc::new(JumpState), 2000);
        manager.enqueue(2, Arc::new(TalkState), 2500);
        manager.enqueue(2, Arc::new(MoveState), 4000);
        manager.enqueue(2, Arc::new(JumpState), 2000);

        manager.update(500);

        while manager.pull_actions().is_some() {}

        let data = manager.to_data(
            convert_actions_from_typed_action_any_map,
            convert_actions_from_typed_any_action_map,
        );
        let str = serde_json::to_string(&data).unwrap();

        let data: ActionManagerData<TestAction> = serde_json::from_str(&str).unwrap();

        let mut new_manager = ActionManager::from_data(data, convert_from_actions).unwrap();

        manager.update(10000);
        new_manager.update(10000);

        let (tick_count, _) = assert_managers(&mut manager, &mut new_manager);
        println!("Tick count = {}", tick_count);
    }
}
