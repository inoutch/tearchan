use serde::{Deserialize, Serialize};

pub mod group;
pub mod group_sync;
pub mod zip;

pub type EntityId = u32;

#[derive(Serialize, Deserialize)]
pub struct Component<T> {
    entity_id: EntityId,
    inner: T,
}

impl<T> Component<T> {
    pub fn new(entity_id: EntityId, inner: T) -> Self {
        Self { entity_id, inner }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
