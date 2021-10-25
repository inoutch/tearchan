use crate::action::Action;
use std::sync::Arc;

#[derive(Debug)]
pub enum ActionResult<T> {
    Start {
        action: Arc<Action<T>>,
    },
    Update {
        action: Arc<Action<T>>,
        current_time: u64,
    },
    End {
        action: Arc<Action<T>>,
    },
    Cancel {
        action: Arc<Action<T>>,
    },
    Enqueue {
        action: Arc<Action<T>>,
    },
}

impl<T> ActionResult<T> {
    pub fn get_start(&self) -> Option<&Arc<Action<T>>> {
        match self {
            ActionResult::Start { action } => Some(action),
            _ => None,
        }
    }

    pub fn get_update(&self) -> Option<(&Arc<Action<T>>, u64)> {
        match self {
            ActionResult::Update {
                action,
                current_time,
            } => Some((action, *current_time)),
            _ => None,
        }
    }

    pub fn get_end(&self) -> Option<&Arc<Action<T>>> {
        match self {
            ActionResult::End { action } => Some(action),
            _ => None,
        }
    }

    pub fn get_cancel(&self) -> Option<&Arc<Action<T>>> {
        match self {
            ActionResult::Cancel { action } => Some(action),
            _ => None,
        }
    }

    pub fn get_enqueue(&self) -> Option<&Arc<Action<T>>> {
        match self {
            ActionResult::Enqueue { action } => Some(action),
            _ => None,
        }
    }
}
