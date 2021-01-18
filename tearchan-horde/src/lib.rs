use crate::action::Action;
use crate::job::result::JobResult;
use tearchan_ecs::component::EntityId;

pub mod action;
pub mod job;

pub trait HordeInterface {
    type ActionState;
    type Job;

    fn on_start(&mut self, action: &Action<Self::ActionState>);

    fn on_update(&mut self, action: &Action<Self::ActionState>, ratio: f32);

    fn on_end(&mut self, action: &Action<Self::ActionState>);

    fn on_first(&self, entity_id: u32) -> Self::Job;

    fn on_next(
        &self,
        entity_id: EntityId,
        job: Self::Job,
    ) -> JobResult<Self::Job, Self::ActionState>;
}