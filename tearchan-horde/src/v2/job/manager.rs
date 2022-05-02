use crate::action::manager::TimeMilliseconds;
use crate::v2::action::manager::{ActionController, ActionManager};
use crate::v2::job::HordeInterface;
use std::collections::VecDeque;

#[derive(Default)]
pub struct JobManager {
    action_manager: ActionManager,
}

impl JobManager {
    pub fn run<T>(&mut self, provider: &mut T, delta: TimeMilliseconds)
    where
        T: HordeInterface,
    {
        while !self.action_manager.get_vacated_entities().is_empty() {
            self.run_actions(provider);
        }
        self.action_manager.update(delta);
        self.run_actions(provider);
    }

    fn run_actions<T>(&mut self, provider: &mut T)
    where
        T: HordeInterface,
    {
        // Loop for each tick
        loop {
            let result_or_none = self.action_manager.pull_actions();
            if let Some(result) = &result_or_none {
                provider.on_change_tick(
                    &result.map,
                    &self.action_manager.validator(),
                );
            }

            let vacated_entities = self.action_manager.pull_vacated_entities();
            if vacated_entities.is_empty() && result_or_none.is_none() {
                break;
            }
            for entity_id in vacated_entities {
                let mut priority = 0;
                let mut job_queue: VecDeque<T::Job> = VecDeque::new();
                job_queue.push_front(provider.on_first(entity_id, priority));

                while let Some(job) = job_queue.pop_front() {
                    let result =
                        provider.on_next(entity_id, job, &mut self.action_manager.controller());
                    if job_queue.is_empty() && !self.action_manager.has_some_actions(entity_id) {
                        // If the jobs and actions cannot be generated from the current job tree,
                        // change the priority and recreate the first job
                        priority += 1;
                        job_queue.push_front(provider.on_first(entity_id, priority));
                        continue;
                    }

                    if let Some(result) = result {
                        job_queue.push_back(result);
                    }
                }
            }
        }

        provider.on_change_time(self.action_manager.pull_updates(), self.action_manager.next_time());
    }

    pub fn action_controller(&mut self) -> ActionController {
        self.action_manager.controller()
    }
}
