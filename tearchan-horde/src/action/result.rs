use crate::action::Action;
use crate::HordeInterface;
use std::sync::Arc;

pub enum ActionResult<T>
where
    T: HordeInterface,
{
    Start {
        action: Arc<Action<T::ActionState>>,
    },
    Update {
        action: Arc<Action<T::ActionState>>,
        current_time: u64,
    },
    End {
        action: Arc<Action<T::ActionState>>,
    },
}
