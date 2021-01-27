use crate::action::{Action, ActionType};
use crate::job::result::JobResult;
use tearchan_ecs::component::EntityId;

pub mod action;
pub mod job;

pub trait HordeInterface {
    type ActionState;
    type Job;

    fn on_action(&mut self, action: &Action<Self::ActionState>, action_type: ActionType);

    fn on_first(&self, entity_id: u32) -> Self::Job;

    fn on_next(
        &self,
        entity_id: EntityId,
        job: Self::Job,
    ) -> JobResult<Self::Job, Self::ActionState>;
}
