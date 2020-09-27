use serde::{Deserialize, Serialize};
use tearchan_core::game::object::GameObjectId;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionStore<TCommonStore> {
    common: TCommonStore,
    pub object_id: GameObjectId,
    pub start_time: u64,
    pub end_time: u64,
}

impl<TCommonStore> ActionStore<TCommonStore> {
    pub fn new(
        common: TCommonStore,
        object_id: GameObjectId,
        start_time: u64,
        end_time: u64,
    ) -> Self {
        ActionStore {
            common,
            object_id,
            start_time,
            end_time,
        }
    }

    pub fn common(&self) -> &TCommonStore {
        &self.common
    }

    pub fn ratio(&self, current_time: u64) -> f32 {
        let a = (self.end_time - self.start_time) as f32;
        let b = (current_time - self.start_time) as f32;
        b / a
    }
}
