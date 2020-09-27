use crate::action_creator::action_creator_context::ActionCreatorContext;
use crate::action_creator::action_creator_store::ActionCreatorStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tearchan_core::game::object::GameObjectId;

#[derive(Debug)]
pub enum ActionCreatorCommand<TActionCreatorCommonStore> {
    Run { store: TActionCreatorCommonStore },
    Rerun,
    Next,
}

#[derive(Serialize, Deserialize)]
pub struct ActionCreatorManagerSnapshot<TActionCreatorCommonStore> {
    contexts: HashMap<GameObjectId, ActionCreatorContext<TActionCreatorCommonStore>>,
}

pub struct ActionCreatorManager<TActionCreatorCommonStore> {
    snapshot: ActionCreatorManagerSnapshot<TActionCreatorCommonStore>,
}

impl<TActionCreatorCommonStore> ActionCreatorManager<TActionCreatorCommonStore> {
    pub fn new() -> Self {
        ActionCreatorManager {
            snapshot: ActionCreatorManagerSnapshot {
                contexts: HashMap::new(),
            },
        }
    }

    pub fn new_with_snapshot(
        snapshot: ActionCreatorManagerSnapshot<TActionCreatorCommonStore>,
    ) -> Self {
        ActionCreatorManager { snapshot }
    }

    pub fn run(
        &mut self,
        object_id: GameObjectId,
        command: ActionCreatorCommand<TActionCreatorCommonStore>,
    ) -> Option<&mut ActionCreatorStore<TActionCreatorCommonStore>> {
        match command {
            ActionCreatorCommand::Run { store } => {
                self.snapshot
                    .contexts
                    .entry(object_id)
                    .or_insert(ActionCreatorContext {
                        store_stack: vec![],
                    });
                let context = self.snapshot.contexts.get_mut(&object_id).unwrap();
                context.store_stack.push(ActionCreatorStore::new(store));
                context.store_stack.last_mut()
            }
            ActionCreatorCommand::Rerun => {
                let context = match self.snapshot.contexts.get_mut(&object_id) {
                    Some(x) => x,
                    None => panic!("Illegal state on rerun"),
                };
                context.store_stack.last_mut()
            }
            ActionCreatorCommand::Next => match self.snapshot.contexts.get_mut(&object_id) {
                Some(context) => {
                    context.store_stack.pop();
                    context.store_stack.last_mut()
                }
                None => None,
            },
        }
    }

    pub fn is_running(&self, object_id: &GameObjectId) -> bool {
        self.snapshot
            .contexts
            .get(object_id)
            .map_or(false, |x| !x.store_stack.is_empty())
    }

    pub fn stop(&mut self, object_id: GameObjectId) {
        self.snapshot.contexts.remove(&object_id);
    }

    pub fn load(&mut self, snapshot: ActionCreatorManagerSnapshot<TActionCreatorCommonStore>) {
        self.snapshot = snapshot;
    }

    pub fn snapshot(&self) -> &ActionCreatorManagerSnapshot<TActionCreatorCommonStore> {
        &self.snapshot
    }
}

#[cfg(test)]
mod test {
    use crate::action_creator::action_creator_manager::{
        ActionCreatorCommand, ActionCreatorManager, ActionCreatorManagerSnapshot,
    };
    use crate::object::live_object_store::LiveObjectId;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "type")]
    enum CommonStore {
        Object1,
        Object2,
    }

    #[test]
    fn test_serialization() {
        let object_id: LiveObjectId = 0;
        let mut action_creator_manager: ActionCreatorManager<CommonStore> =
            ActionCreatorManager::new();
        action_creator_manager.run(
            object_id,
            ActionCreatorCommand::Run {
                store: CommonStore::Object1,
            },
        );
        action_creator_manager.run(
            object_id,
            ActionCreatorCommand::Run {
                store: CommonStore::Object2,
            },
        );
        let string = serde_json::to_string(action_creator_manager.snapshot()).unwrap();
        let snapshot: ActionCreatorManagerSnapshot<CommonStore> =
            serde_json::from_str(&string).unwrap();
        let mut restored_action_creator_manager = ActionCreatorManager::new_with_snapshot(snapshot);

        assert!(restored_action_creator_manager.is_running(&object_id));
        let ret = restored_action_creator_manager.run(object_id, ActionCreatorCommand::Rerun);
        assert!(matches!(ret.unwrap().store_mut(), CommonStore::Object2));

        let ret = restored_action_creator_manager.run(object_id, ActionCreatorCommand::Next);
        assert!(matches!(ret.unwrap().store_mut(), CommonStore::Object1));
    }
}
