use crate::action::manager::TimeMilliseconds;

pub struct ActionContext {
    pub last_time: TimeMilliseconds,
    pub state_len: usize, // running and pending state length of entity
}
