use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionCreatorStore<TCommonStore> {
    store: TCommonStore,
}

impl<TCommonStore> ActionCreatorStore<TCommonStore> {
    pub fn new(store: TCommonStore) -> Self {
        ActionCreatorStore { store }
    }

    pub fn store(&self) -> &TCommonStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut TCommonStore {
        &mut self.store
    }
}
