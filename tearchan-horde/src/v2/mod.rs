pub mod action;
pub mod job;
pub use serde;

pub type Lazy<T> = once_cell::sync::Lazy<T>;
pub type Tick = u64;

#[macro_export]
macro_rules! define_actions {
    ($name:tt, $(($member:tt, $struct:tt)),*) => {
        type Mapper = std::collections::HashMap<std::any::TypeId, fn(any: &$crate::v2::action::collection::ActionAnyVec, validator: &$crate::v2::action::manager::ActionSessionValidator) -> Vec<$crate::v2::action::Action<$name>>>;
        static MAPPER: $crate::v2::Lazy<Mapper> = $crate::v2::Lazy::new(|| {
            let mut map: Mapper = std::collections::HashMap::new();
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

        #[derive(Clone, Debug, $crate::v2::serde::Serialize, $crate::v2::serde::Deserialize)]
        enum $name {
            $(
                $member(std::sync::Arc<$struct>),
            )*
        }

        #[allow(dead_code)]
        fn covert_to_actions(
            map: &$crate::v2::action::collection::TypedActionAnyMap,
            validator: &$crate::v2::action::manager::ActionSessionValidator
        ) -> Vec<$crate::v2::action::Action<$name>> {
            map.iter()
                .filter_map(|(key, value)| MAPPER.get(&key).map(|f| f(value, validator)))
                .flatten()
                .collect()
        }

        #[allow(dead_code)]
        fn convert_from_actions(actions: &Vec<$crate::v2::action::Action<$name>>) -> $crate::v2::action::collection::TypedActionAnyMap {
            let mut map = $crate::v2::action::collection::TypedActionAnyMap::default();
            for action in actions {
                match action.raw() {
                $(
                    $name::$member(state) => {
                        map.push(action.replace(state.clone()), $crate::v2::action::VALID_SESSION_ID);
                    },
                )*
                }
            }
            map
        }
    }
}

#[cfg(test)]
mod test {
    use crate::v2::action::collection::TypedActionAnyMap;
    use crate::v2::action::manager::ActionSessionValidator;
    use crate::v2::action::{Action, ActionType};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct MoveState;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct JumpState;

    define_actions!(TestAction, (Move, MoveState), (Jump, JumpState));

    #[test]
    fn test() {
        let validator = ActionSessionValidator::default();
        let mut collections = TypedActionAnyMap::default();
        collections.push(
            Action::new(Arc::new(MoveState), 1, ActionType::Start { tick: 0 }),
            0,
        );

        let move_states = collections.get::<MoveState>(&validator);
        let mut actions = covert_to_actions(&collections, &validator);
        actions.sort_by_key(|x| x.entity_id());

        assert_eq!(move_states.unwrap().len(), 1);
        assert_eq!(actions.len(), 1);

        let collections = convert_from_actions(&actions);
        let move_states = collections.get::<MoveState>(&validator);
        assert_eq!(move_states.unwrap().len(), 1);
    }
}
