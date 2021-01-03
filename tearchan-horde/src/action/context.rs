use crate::action::manager::TimeMilliseconds;

pub struct ActionContext {
    pub last_time: TimeMilliseconds,
    pub state_len: usize,
}
