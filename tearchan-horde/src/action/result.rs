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
