#![feature(map_first_last)]

use crate::action::manager::{ActionController, ActionServerReader};
use crate::action::Action;
use crate::job::result::{JobResult, ProgressState};
use std::sync::Arc;
use tearchan_ecs::component::EntityId;

pub mod action;
pub mod job;
pub mod v2;

pub trait HordeInterface {
    type ActionState;
    type Job;

    fn on_start(
        &mut self,
        action: &Action<Self::ActionState>,
        controller: &mut ActionController<Self::ActionState>,
    );

    fn on_update(
        &mut self,
        action: &Action<Self::ActionState>,
        ratio: f32,
        controller: &mut ActionController<Self::ActionState>,
    );

    fn on_end(
        &mut self,
        action: &Action<Self::ActionState>,
        controller: &mut ActionController<Self::ActionState>,
    );

    fn on_enqueue(
        &mut self,
        action: &Action<Self::ActionState>,
        controller: &mut ActionController<Self::ActionState>,
    );

    fn on_first(
        &self,
        entity_id: EntityId,
        priority: u32,
        reader: &ActionServerReader<Self::ActionState>,
    ) -> Option<Self::Job>;

    fn on_next(
        &self,
        entity_id: EntityId,
        job: Self::Job,
        reader: &ActionServerReader<Self::ActionState>,
    ) -> JobResult<Self::Job, Self::ActionState>;

    fn on_send(&self, _action: Arc<Action<Self::ActionState>>) {}

    fn on_cancel(
        &mut self,
        action: &Action<Self::ActionState>,
        controller: &mut ActionController<Self::ActionState>,
    );

    fn on_change_time(
        &mut self,
        reader: &ActionServerReader<Self::ActionState>,
    ) -> Vec<(EntityId, ProgressState<Self::ActionState>)>;
}
