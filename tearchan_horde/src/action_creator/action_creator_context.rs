use crate::action_creator::action_creator_store::ActionCreatorStore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionCreatorContext<TCommonActionCreatorStore> {
    pub store_stack: Vec<ActionCreatorStore<TCommonActionCreatorStore>>, // The last store is current
}
