use crate::action::manager::TimeMilliseconds;
use crate::v2::Tick;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tearchan_ecs::component::EntityId;

pub mod collection;
pub mod manager;

pub const VALID_SESSION_ID: ActionSessionId = 0;

pub type ActionSessionId = u64;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ActionType {
    Start {
        tick: Tick,
    },
    Update {
        start: TimeMilliseconds,
        end: TimeMilliseconds,
    },
    End {
        tick: Tick,
    },
}

impl ActionType {
    pub fn tick(&self) -> Option<Tick> {
        match self {
            ActionType::Start { tick, .. } => Some(*tick),
            ActionType::Update { .. } => None,
            ActionType::End { tick, .. } => Some(*tick),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action<T> {
    raw: T,
    entity_id: EntityId,
    ty: ActionType,
}

impl<T> Action<T> {
    pub fn new(raw: T, entity_id: EntityId, ty: ActionType) -> Self {
        Self { raw, entity_id, ty }
    }

    pub fn tick(&self) -> Option<Tick> {
        self.ty.tick()
    }
}

pub type ArcAction<T> = Action<Arc<T>>;

impl<T> Action<T> {
    pub fn raw(&self) -> &T {
        &self.raw
    }

    pub fn replace<U>(&self, raw: U) -> Action<U> {
        Action {
            raw,
            entity_id: self.entity_id,
            ty: self.ty,
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}
