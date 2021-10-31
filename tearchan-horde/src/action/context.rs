use crate::action::manager::TimeMilliseconds;

pub struct ActionContext {
    pub last_time: TimeMilliseconds, // The last time of stacking all actions
    pub running_end_time: TimeMilliseconds, // The end time of running action
}
