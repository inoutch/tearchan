use crate::action::manager::TimeMilliseconds;
use crate::v2::action::collection::{
    ActionMeta, TypedActionAnyMap, TypedAnyActionVecGroupedByEntityId,
};
use crate::v2::action::{Action, ActionSessionId, ActionType, VALID_SESSION_ID};
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::sync::Arc;
use tearchan_ecs::component::EntityId;

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
struct ActionQueueItem {
    map: TypedActionAnyMap,
    events: VecDeque<(EntityId, Event)>,
    cancels: BTreeSet<EntityId>,
}

pub struct PullActionResult {
    pub map: TypedActionAnyMap,
    pub cancels: BTreeSet<EntityId>,
}

#[derive(Default)]
pub struct ActionManager {
    next_time: TimeMilliseconds,
    next_tick: Tick,
    current_tick: Tick,
    actions: BTreeMap<Tick, ActionQueueItem>,
    update_actions: TypedAnyActionVecGroupedByEntityId,
    contexts: HashMap<EntityId, ActionContext>,
    vacated_entities: BTreeSet<EntityId>,
}

impl ActionManager {
    pub fn attach(&mut self, entity_id: EntityId) {
        debug_assert!(!self.contexts.contains_key(&entity_id));

        self.contexts.insert(
            entity_id,
            ActionContext {
                last_tick: self.current_tick,
                running_end_tick: self.current_tick,
                session_id: 1,
                session_expired_at: self.current_tick,
            },
        );
    }

    pub fn detach(&mut self, entity_id: EntityId) {
        self.contexts.remove(&entity_id);
    }

    pub fn update(&mut self, delta: TimeMilliseconds) {
        debug_assert!(self.vacated_entities.is_empty());

        self.next_time += delta;
        self.next_tick = self.next_time / TICK_DURATION;
    }

    pub fn enqueue<T>(&mut self, entity_id: EntityId, raw: Arc<T>, duration: TimeMilliseconds)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let context = self
            .contexts
            .get_mut(&entity_id)
            .unwrap_or_else(|| panic!("entity of {} is not attached", entity_id));
        debug_assert!(self.current_tick <= context.running_end_tick);
        debug_assert!(context.running_end_tick <= context.last_tick);

        let start_tick = context.last_tick;
        let end_tick = start_tick + duration / TICK_DURATION;
        {
            let item = self
                .actions
                .entry(start_tick)
                .or_insert_with(Default::default);
            item.map.push(
                Action {
                    raw: Arc::clone(&raw),
                    entity_id,
                    ty: ActionType::Start { tick: start_tick },
                },
                context.session_id,
            );
            item.events.push_back((
                entity_id,
                Event::Started {
                    type_id,
                    update_action: Box::new(Action {
                        raw: Arc::clone(&raw),
                        entity_id,
                        ty: ActionType::Update {
                            start: start_tick * TICK_DURATION,
                            end: end_tick * TICK_DURATION,
                        },
                    }),
                    running_end_tick: end_tick,
                },
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
                    ty: ActionType::End { tick: end_tick },
                },
                context.session_id,
            );
            item.events.push_back((entity_id, Event::Ended));
        }
        context.last_tick = end_tick;
    }

    pub fn pull_actions(&mut self) -> Option<PullActionResult> {
        let (tick, mut item) = self.actions.pop_first()?;
        if tick > self.next_tick {
            self.current_tick = self.next_tick;
            self.actions.insert(tick, item);
            return None;
        }

        self.current_tick = tick;

        while let Some((entity_id, event)) = item.events.pop_front() {
            let context = self.contexts.get_mut(&entity_id).unwrap();
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

    pub fn pull_updates(&self) -> &TypedAnyActionVecGroupedByEntityId {
        &self.update_actions
    }

    pub fn pull_vacated_entities(&mut self) -> BTreeSet<EntityId> {
        std::mem::take(&mut self.vacated_entities)
    }

    pub fn cancel(&mut self, entity_id: EntityId, immediate: bool) {
        self.update_actions.remove(entity_id);

        let context = self.contexts.get_mut(&entity_id).unwrap();
        context.session_id += 1;
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
        item.events.push_back((entity_id, Event::Canceled));
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
}

#[cfg(test)]
#[macro_use]
mod test {
    use crate::define_actions;
    use crate::v2::action::manager::{
        ActionManager, ActionSessionValidator, PullActionResult, Tick,
    };
    use crate::v2::action::ArcAction;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct MoveState;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct JumpState;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct TalkState;

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

    fn snapshot(manager: &mut ActionManager) {
        while let Some(actions) = manager.pull_actions() {
            insta::assert_debug_snapshot!(SnapshotResult {
                current_tick: manager.current_tick(),
                tag: "changes",
                move_actions: actions.map.get(&manager.validator()),
                jump_actions: actions.map.get(&manager.validator()),
                talk_actions: actions.map.get(&manager.validator())
            });
        }
        snapshot_update(manager);
    }

    fn snapshot_update(manager: &mut ActionManager) {
        let actions = manager.pull_updates();

        insta::assert_debug_snapshot!(SnapshotResult {
            current_tick: manager.current_tick(),
            tag: "updates",
            move_actions: actions.get(),
            jump_actions: actions.get(),
            talk_actions: actions.get()
        });
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

        snapshot(&mut manager);

        // --- Tick: 3 ---- //
        manager.update(200);

        snapshot(&mut manager);

        // --- Tick: 7 ---- //
        manager.update(300);

        snapshot(&mut manager);

        // --- Tick: 60
        manager.update(3328);

        snapshot(&mut manager);
    }

    #[test]
    fn test_vacated_entities() {
        let mut manager = ActionManager::default();
        manager.attach(1);
        manager.attach(2);

        manager.enqueue(1, Arc::new(MoveState), 1000); // tick: 15
        manager.enqueue(2, Arc::new(MoveState), 500); // tick: 7

        assert!(manager.pull_vacated_entities().is_empty());

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
    fn test_zero_time_action() {}

    #[test]
    fn test_detach() {}

    #[test]
    fn test_serialization() {}
}
