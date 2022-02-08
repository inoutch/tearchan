use crate::action::manager::TimeMilliseconds;

pub type ProgressState<ActionState> = (ActionState, TimeMilliseconds);

#[derive(Debug)]
pub struct JobResult<Job, ActionState> {
    pub creators: Vec<Job>,
    pub states: Vec<ProgressState<ActionState>>,
}

impl<ActionCreator, ActionState> Default for JobResult<ActionCreator, ActionState> {
    fn default() -> Self {
        JobResult {
            creators: vec![],
            states: vec![],
        }
    }
}
