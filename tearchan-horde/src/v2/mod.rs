pub mod action;
pub mod job;
pub use serde;

pub type Lazy<T> = once_cell::sync::Lazy<T>;
pub type Tick = u64;

#[macro_export]
macro_rules! define_actions {
    ($name:tt, $(($member:tt, $struct:tt)),*) => {
        type Mapper0 = std::collections::HashMap<std::any::TypeId, fn(any: &$crate::v2::action::collection::ActionAnyVec, validator: &$crate::v2::action::manager::ActionSessionValidator) -> Vec<$crate::v2::action::Action<$name>>>;
        type Mapper1 = std::collections::HashMap<std::any::TypeId, fn(any: &$crate::v2::action::collection::AnyActionVec) -> Vec<$crate::v2::action::Action<$name>>>;
        static MAPPER0: $crate::v2::Lazy<Mapper0> = $crate::v2::Lazy::new(|| {
            let mut map: Mapper0 = std::collections::HashMap::new();
            $(
            map.insert(std::any::TypeId::of::<$struct>(), |vec, validator| {
                vec.cast::<$struct>(validator)
                    .iter()
                    .map(|action| action.replace($name::$member(action.raw().clone())))
                    .collect::<Vec<_>>()
            });
            )*
            map
        });
        static MAPPER1: $crate::v2::Lazy<Mapper1> = $crate::v2::Lazy::new(|| {
            let mut map: Mapper1 = std::collections::HashMap::new();
            $(
            map.insert(std::any::TypeId::of::<$struct>(), |vec| {
                vec.cast::<$struct>()
                    .iter()
                    .map(|action| action.replace($name::$member(action.raw().clone())))
                    .collect::<Vec<_>>()
            });
            )*
            map
        });

        #[derive(Clone, Debug, $crate::v2::serde::Serialize, $crate::v2::serde::Deserialize)]
        enum $name {
            $(
                $member(std::sync::Arc<$struct>),
            )*
        }

        #[allow(dead_code)]
        fn convert_actions_from_typed_action_any_map(
            map: &$crate::v2::action::collection::TypedActionAnyMap,
            validator: &$crate::v2::action::manager::ActionSessionValidator
        ) -> Vec<$crate::v2::action::Action<$name>> {
            map.iter()
                .filter_map(|(key, value)| MAPPER0.get(&key).map(|f| f(value, validator)))
                .flatten()
                .collect()
        }

        #[allow(dead_code)]
        fn convert_actions_from_typed_any_action_map(
            map: &$crate::v2::action::collection::TypedAnyActionMapGroupedByEntityId
        ) -> Vec<$crate::v2::action::Action<$name>> {
            map.iter()
                .filter_map(|(key, value)| MAPPER1.get(&key).map(|f| f(value)))
                .flatten()
                .collect()
        }

        #[allow(dead_code)]
        fn convert_from_actions(action: $crate::v2::action::Action<$name>, converter: &mut $crate::v2::action::manager::ActionManagerConverter) {
            match action.raw() {
            $(
                $name::$member(state) => {
                    converter.load(action.replace(Arc::clone(state)));
                },
            )*
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::v2::action::collection::{TypedActionAnyMap, TypedAnyActionMapGroupedByEntityId};
    use crate::v2::action::manager::ActionSessionValidator;
    use crate::v2::action::{Action, ActionSessionId, ActionType};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct MoveState;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct JumpState;

    define_actions!(TestAction, (Move, MoveState), (Jump, JumpState));

    #[test]
    fn test_macro_0() {
        let validator = ActionSessionValidator::default();
        let mut collections = TypedActionAnyMap::default();
        collections.push(
            Action::new(
                Arc::new(MoveState),
                1,
                ActionType::Start {
                    tick: 0,
                    start: 0,
                    end: 0,
                },
            ),
            ActionSessionId::default(),
        );

        let move_states = collections.get::<MoveState>(&validator);
        let mut actions = convert_actions_from_typed_action_any_map(&collections, &validator);
        actions.sort_by_key(|x| x.entity_id());

        assert_eq!(move_states.unwrap().len(), 1);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_macro_1() {
        let mut collections = TypedAnyActionMapGroupedByEntityId::default();
        collections.insert(
            0,
            Action::new(
                Arc::new(MoveState),
                1,
                ActionType::Start {
                    tick: 0,
                    start: 0,
                    end: 0,
                },
            ),
        );

        let move_states = collections.get::<MoveState>();
        let mut actions = convert_actions_from_typed_any_action_map(&collections);
        actions.sort_by_key(|x| x.entity_id());

        assert_eq!(move_states.unwrap().len(), 1);
        assert_eq!(actions.len(), 1);
    }
}
