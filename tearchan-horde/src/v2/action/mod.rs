use crate::action::manager::TimeMilliseconds;
use crate::v2::Tick;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tearchan_ecs::component::EntityId;

pub mod collection;
pub mod manager;

pub const VALID_SESSION_ID: ActionSessionId = ActionSessionId(0);

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ActionSessionId(u128);

impl Default for ActionSessionId {
    fn default() -> Self {
        ActionSessionId(1)
    }
}

impl ActionSessionId {
    pub fn next(&self) -> Self {
        ActionSessionId(self.0 + 1)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ActionType {
    Start {
        start: Tick,
        end: Tick,
    },
    Update {
        start: TimeMilliseconds,
        end: TimeMilliseconds,
    },
    End {
        start: Tick,
        end: Tick,
    },
}

impl ActionType {
    pub fn tick(&self) -> Option<Tick> {
        match self {
            ActionType::Start { start, .. } => Some(*start),
            ActionType::Update { .. } => None,
            ActionType::End { end, .. } => Some(*end),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

    pub fn ty(&self) -> &ActionType {
        &self.ty
    }
}
