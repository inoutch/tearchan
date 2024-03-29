use serde::{Deserialize, Serialize};

pub mod group;
pub mod group_sync;
pub mod resource_sync;
pub mod zip;

pub type EntityId = u64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Component<T> {
    #[serde(rename = "entityId")]
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
    
    pub fn into_inner(self) -> T {
        self.inner
    }
}
