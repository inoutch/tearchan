use crate::action::manager::TimeMilliseconds;
use crate::v2::action::collection::{TypedAnyActionMap, TypedAnyActionMapGroupedByEntityId};
use crate::v2::action::manager::ActionController;
use crate::v2::job::manager::JobController;
use tearchan_ecs::component::EntityId;

pub mod manager;

pub trait HordeInterface {
    type Job: Clone;

    fn on_change_tick(&mut self, map: &TypedAnyActionMap, controller: JobController<Self::Job>);

    fn on_change_time(&mut self, map: &TypedAnyActionMapGroupedByEntityId, time: TimeMilliseconds);

    fn on_cancel_job(&mut self, entity_id: EntityId, jobs: Vec<Self::Job>);

    fn on_first(&self, entity_id: EntityId, priority: u32) -> Self::Job;

    fn on_next(
        &self,
        entity_id: EntityId,
        job: Self::Job,
        controller: &mut ActionController,
    ) -> Option<Self::Job>;
}
