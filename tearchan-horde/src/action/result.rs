use crate::action::Action;
use crate::HordeInterface;
use std::rc::Rc;

pub enum ActionResult<T>
where
    T: HordeInterface,
{
    Start {
        action: Rc<Action<T::ActionState>>,
    },
    Update {
        action: Rc<Action<T::ActionState>>,
        current_time: u64,
    },
    End {
        action: Rc<Action<T::ActionState>>,
    },
}
