use crate::action::Action;
use std::sync::Arc;

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
}

impl<T> ActionResult<T> {
    pub fn get_start(&self) -> Option<&Arc<Action<T>>> {
        match self {
            ActionResult::Start { action } => Some(action),
            ActionResult::Update { .. } => None,
            ActionResult::End { .. } => None,
        }
    }

    pub fn get_update(&self) -> Option<(&Arc<Action<T>>, u64)> {
        match self {
            ActionResult::Start { .. } => None,
            ActionResult::Update {
                action,
                current_time,
            } => Some((action, *current_time)),
            ActionResult::End { .. } => None,
        }
    }

    pub fn get_end(&self) -> Option<&Arc<Action<T>>> {
        match self {
            ActionResult::Start { .. } => None,
            ActionResult::Update { .. } => None,
            ActionResult::End { action } => Some(action),
        }
    }
}
