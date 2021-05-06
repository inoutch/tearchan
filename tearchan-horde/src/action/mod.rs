use crate::action::manager::TimeMilliseconds;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tearchan_ecs::component::EntityId;

pub mod context;
pub mod manager;
pub mod result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Action<T> {
    #[serde(rename = "entityId")]
    entity_id: EntityId,
    #[serde(rename = "startTime")]
    start_time: TimeMilliseconds,
    #[serde(rename = "endTime")]
    end_time: TimeMilliseconds,
    inner: Arc<T>,
}

impl<T> Action<T> {
    pub fn new(
        entity_id: EntityId,
        start: TimeMilliseconds,
        end: TimeMilliseconds,
        inner: T,
    ) -> Action<T> {
        Action {
            entity_id,
            start_time: start,
            end_time: end,
            inner: Arc::new(inner),
        }
    }
}

impl<T> Action<T> {
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn start_time(&self) -> TimeMilliseconds {
        self.start_time
    }

    pub fn end_time(&self) -> TimeMilliseconds {
        self.end_time
    }

    pub fn inner(&self) -> &Arc<T> {
        &self.inner
    }
}
