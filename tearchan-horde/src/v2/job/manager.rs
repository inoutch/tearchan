use crate::action::manager::TimeMilliseconds;
use crate::v2::action::collection::{TypedActionAnyMap, TypedAnyActionMapGroupedByEntityId};
use crate::v2::action::manager::{
    ActionManager, ActionManagerConverter, ActionManagerData, ActionManagerError,
    ActionSessionValidator,
};
use crate::v2::action::Action;
use crate::v2::job::HordeInterface;
use crate::v2::Tick;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tearchan_ecs::component::EntityId;
use tearchan_ecs::entity::manager::ENTITY_REMAPPER;

pub struct JobManager<T: HordeInterface> {
    action_manager: ActionManager,
    jobs: HashMap<EntityId, Vec<T::Job>>,
}

impl<T> Default for JobManager<T>
where
    T: HordeInterface,
{
    fn default() -> Self {
        JobManager {
            action_manager: Default::default(),
            jobs: Default::default(),
        }
    }
}

impl<T> JobManager<T>
where
    T: HordeInterface,
{
    pub fn run(&mut self, provider: &mut T, delta: TimeMilliseconds) {
        while !self.action_manager.get_vacated_entities().is_empty() {
            self.run_actions(provider);
        }
        self.action_manager.update(delta);
        self.run_actions(provider);
    }

    #[inline]
    pub fn attach(&mut self, entity_id: EntityId) {
        self.action_manager.attach(entity_id);
        self.jobs.insert(entity_id, Vec::new());
    }

    #[inline]
    pub fn detach(&mut self, entity_id: EntityId) {
        self.action_manager.detach(entity_id);
        self.jobs.remove(&entity_id);
    }

    #[inline]
    pub fn interrupt<U>(&mut self, entity_id: EntityId, raw: Arc<U>, duration: TimeMilliseconds)
    where
        U: 'static,
    {
        self.action_manager.interrupt(entity_id, raw, duration);
    }

    pub fn to_data<U>(
        &self,
        converter0: fn(&TypedActionAnyMap, &ActionSessionValidator) -> Vec<Action<U>>,
        converter1: fn(&TypedAnyActionMapGroupedByEntityId) -> Vec<Action<U>>,
    ) -> JobManagerData<U, T::Job> {
        JobManagerData {
            action_manager_data: self.action_manager.to_data(converter0, converter1),
            jobs: self.jobs.clone(),
        }
    }

    pub fn load_data<U>(
        &mut self,
        mut data: JobManagerData<U, T::Job>,
        converter: fn(action: Action<U>, manager: &mut ActionManagerConverter),
    ) {
        self.action_manager
            .load_data(data.action_manager_data, converter);
        for (entity_id, job) in data.jobs.drain() {
            self.jobs.insert(ENTITY_REMAPPER.remap(entity_id), job);
        }
    }

    pub fn from_data<U>(
        data: JobManagerData<U, T::Job>,
        converter: fn(action: Action<U>, manager: &mut ActionManagerConverter),
    ) -> Result<Self, JobManagerError> {
        let action_manager = ActionManager::from_data(data.action_manager_data, converter)
            .map_err(JobManagerError::ActionManagerError)?;
        for (entity_id, _) in data.jobs.iter() {
            if !action_manager.has_some_actions(*entity_id) {
                return Err(JobManagerError::NotFoundEntity(*entity_id));
            }
        }
        Ok(JobManager {
            action_manager,
            jobs: data.jobs,
        })
    }

    #[inline]
    pub fn current_tick(&self) -> Tick {
        self.action_manager.current_tick()
    }

    fn run_actions(&mut self, provider: &mut T) {
        // Loop for each tick
        loop {
            let result_or_none = self.action_manager.pull_actions();
            if let Some(result) = &result_or_none {
                provider.on_change_tick(&result.map, &self.action_manager.validator());

                for entity_id in result.cancels.iter() {
                    provider.on_cancel_job(
                        *entity_id,
                        std::mem::take(self.jobs.get_mut(entity_id).unwrap()),
                    );
                }
            }

            let vacated_entities = self.action_manager.pull_vacated_entities();
            if vacated_entities.is_empty() && result_or_none.is_none() {
                break;
            }
            for entity_id in vacated_entities {
                let mut priority = 0;
                let mut job_queue: VecDeque<T::Job> = VecDeque::new();
                self.jobs.get_mut(&entity_id).unwrap().clear();
                job_queue.push_front(provider.on_first(entity_id, priority));

                while let Some(job) = job_queue.pop_front() {
                    self.jobs.get_mut(&entity_id).unwrap().push(job.clone());

                    let result =
                        provider.on_next(entity_id, job, &mut self.action_manager.controller());
                    if job_queue.is_empty() && !self.action_manager.has_some_actions(entity_id) {
                        // If the jobs and actions cannot be generated from the current job tree,
                        // change the priority and recreate the first job
                        priority += 1;
                        self.jobs.get_mut(&entity_id).unwrap().clear();
                        job_queue.push_front(provider.on_first(entity_id, priority));
                        continue;
                    }

                    if let Some(result) = result {
                        job_queue.push_back(result);
                    }
                }
            }
        }

        provider.on_change_time(
            self.action_manager.pull_updates(),
            self.action_manager.next_time(),
        );
    }
}

pub struct JobController<'a> {
    action_manager: &'a mut ActionManager,
}

impl<'a> JobController<'a> {
    #[inline]
    pub fn enqueue<T>(&mut self, entity_id: EntityId, raw: Arc<T>, duration: TimeMilliseconds)
    where
        T: 'static,
    {
        self.action_manager.enqueue(entity_id, raw, duration);
    }
}

#[derive(Serialize, Deserialize)]
pub struct JobManagerData<T, U> {
    action_manager_data: ActionManagerData<T>,
    jobs: HashMap<EntityId, Vec<U>>,
}

#[derive(Debug)]
pub enum JobManagerError {
    ActionManagerError(ActionManagerError),
    NotFoundEntity(EntityId),
}
