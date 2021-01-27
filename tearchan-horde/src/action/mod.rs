use crate::action::manager::TimeMilliseconds;
use std::rc::Rc;
use tearchan_ecs::component::EntityId;

pub mod context;
pub mod manager;
pub mod result;

#[derive(Debug)]
pub struct Action<T> {
    entity_id: EntityId,
    start_time: TimeMilliseconds,
    end_time: TimeMilliseconds,
    inner: Rc<T>,
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
            inner: Rc::new(inner),
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

    pub fn inner(&self) -> &Rc<T> {
        &self.inner
    }
}

#[derive(Debug)]
pub enum ActionType {
    Start,
    Update { ratio: f32 },
    End,
}
