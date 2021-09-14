use crate::action::manager::{ActionController, ActionManager, TimeMilliseconds};
use crate::action::result::ActionResult;
use crate::HordeInterface;
use std::collections::VecDeque;
use std::ops::Deref;
use std::option::Option::Some;
use std::sync::Arc;
use tearchan_ecs::component::EntityId;

pub struct JobManager<T>
where
    T: HordeInterface,
{
    action_manager: ActionManager<T::ActionState>,
}

impl<T> Default for JobManager<T>
where
    T: HordeInterface,
{
    fn default() -> Self {
        JobManager {
            action_manager: ActionManager::default(),
        }
    }
}

impl<T> JobManager<T>
where
    T: HordeInterface,
{
    pub fn run(&mut self, provider: &mut T, elapsed_time: TimeMilliseconds) {
        self.action_manager.update(elapsed_time);

        loop {
            self.update_action(provider);

            let mut is_changed = false;
            let entity_ids = self.action_manager.clean_pending_entity_ids();
            for entity_id in entity_ids {
                let mut priority = 0;
                let mut job_queue: VecDeque<T::Job> = VecDeque::new();
                job_queue.push_front(get_until_some_priority_is_returned(
                    |entity_id, priority| {
                        provider.on_first(entity_id, priority, &self.action_manager.reader())
                    },
                    entity_id,
                    &mut priority,
                ));

                while let Some(job) = job_queue.pop_front() {
                    let result = provider.on_next(entity_id, job, &self.action_manager.reader());
                    if job_queue.is_empty()
                        && result.states.is_empty()
                        && result.creators.is_empty()
                    {
                        // If the jobs and actions cannot be generated from the current job tree,
                        // change the priority and recreate the first job
                        priority += 1;
                        job_queue.push_front(get_until_some_priority_is_returned(
                            |entity_id, priority| {
                                provider.on_first(
                                    entity_id,
                                    priority,
                                    &self.action_manager.reader(),
                                )
                            },
                            entity_id,
                            &mut priority,
                        ));
                        continue;
                    }

                    // Update actions
                    is_changed |= !result.states.is_empty();

                    // Add action creators
                    for creator in result.creators.into_iter().rev() {
                        job_queue.push_front(creator);
                    }

                    // Add actions
                    for action in self.action_manager.push_states(entity_id, result.states) {
                        provider.on_enqueue(action.as_ref(), &mut self.action_manager.controller());
                    }
                }
            }

            if !is_changed {
                break;
            }
        }
    }

    pub fn update_action(&mut self, provider: &mut T) {
        let mut results = self.action_manager.pull();
        let mut controller = self.action_manager.controller();

        while let Some(result) = results.pop_first_back() {
            match result {
                ActionResult::Start { action } => {
                    provider.on_send(Arc::clone(&action));
                    provider.on_start(action.deref(), &mut controller);
                }
                ActionResult::Update {
                    action,
                    current_time,
                } => {
                    let duration = action.end_time() - action.start_time();
                    let ratio = (current_time - action.start_time()) as f32 / duration as f32;
                    provider.on_update(action.deref(), ratio, &mut controller);
                }
                ActionResult::End { action } => {
                    provider.on_end(action.deref(), &mut controller);
                }
            }
        }
    }

    pub fn action_controller(&mut self) -> ActionController<T::ActionState> {
        self.action_manager.controller()
    }

    pub fn action_manager(&self) -> &ActionManager<T::ActionState> {
        &self.action_manager
    }

    pub fn action_manager_mut(&mut self) -> &mut ActionManager<T::ActionState> {
        &mut self.action_manager
    }
}

impl<T: HordeInterface> From<ActionManager<T::ActionState>> for JobManager<T> {
    fn from(action_manager: ActionManager<T::ActionState>) -> Self {
        JobManager { action_manager }
    }
}

fn get_until_some_priority_is_returned<F, R>(mut f: F, entity_id: EntityId, priority: &mut u32) -> R
where
    F: FnMut(EntityId, u32) -> Option<R>,
{
    loop {
        if let Some(v) = f(entity_id, *priority) {
            return v;
        }
        *priority += 1;

        debug_assert!(*priority < 1000, "Priority has been exceeded");
    }
}

#[cfg(test)]
mod test {
    use crate::action::manager::{ActionController, ActionReader};
    use crate::action::Action;
    use crate::job::manager::JobManager;
    use crate::job::result::JobResult;
    use crate::HordeInterface;
    use tearchan_ecs::component::group::ComponentGroup;
    use tearchan_ecs::component::EntityId;
    use tearchan_util::id_manager::IdManager;

    #[derive(Debug)]
    enum Kind {
        Dog,
        Cat,
        Human,
    }

    #[derive(Debug)]
    enum CustomActionState {
        Move { position: Position },
        Sleep,
        Eat { food_name: &'static str },
        Job { salary: u32 },
    }

    #[derive(Debug)]
    struct Position((u32, u32));

    enum CustomActionCreator {
        EatLunch {
            position: Position,
            food_name: &'static str,
        },
        EatFood {
            food_name: &'static str,
        },
        MoveTo {
            position: Position,
        },
        Sleep,
        Job {
            salary: u32,
        },
        Work {
            salary: u32,
            position: Position,
        },
        #[allow(dead_code)]
        Invalid,
    }

    struct CustomGame {
        pub entity_id_manager: IdManager<EntityId>,
        pub kind_components: ComponentGroup<Kind>,
        pub name_components: ComponentGroup<String>,
        pub position_components: ComponentGroup<Position>,
    }

    impl HordeInterface for CustomGame {
        type ActionState = CustomActionState;
        type Job = CustomActionCreator;

        fn on_start(
            &mut self,
            _action: &Action<Self::ActionState>,
            _controller: &mut ActionController<CustomActionState>,
        ) {
            println!("start  : {:?}", _action);
        }

        fn on_update(
            &mut self,
            _action: &Action<Self::ActionState>,
            _ratio: f32,
            _controller: &mut ActionController<CustomActionState>,
        ) {
            println!("update : {:?}", _action);
        }

        fn on_end(
            &mut self,
            _action: &Action<Self::ActionState>,
            _controller: &mut ActionController<CustomActionState>,
        ) {
            println!("end    : {:?}", _action);
        }

        fn on_enqueue(
            &mut self,
            action: &Action<Self::ActionState>,
            _controller: &mut ActionController<CustomActionState>,
        ) {
            println!("queue  : {:?}", action);
        }

        fn on_first(
            &self,
            entity_id: u32,
            _priority: u32,
            _reader: &ActionReader<Self::ActionState>,
        ) -> Option<Self::Job> {
            let kind = self.kind_components.get(entity_id).unwrap();
            Some(match kind {
                Kind::Dog => CustomActionCreator::EatLunch {
                    position: Position((100, 200)),
                    food_name: "dog food",
                },
                Kind::Cat => CustomActionCreator::Sleep,
                Kind::Human => CustomActionCreator::Work {
                    position: Position((100, 200)),
                    salary: 200, // $200
                },
            })
        }

        fn on_next(
            &self,
            _entity_id: u32,
            job: Self::Job,
            _reader: &ActionReader<Self::ActionState>,
        ) -> JobResult<Self::Job, Self::ActionState> {
            let mut result = JobResult::default();
            match job {
                CustomActionCreator::EatLunch {
                    position,
                    food_name,
                } => {
                    result
                        .creators
                        .push(CustomActionCreator::MoveTo { position });
                    result
                        .creators
                        .push(CustomActionCreator::EatFood { food_name });
                    return result;
                }
                CustomActionCreator::MoveTo { position } => {
                    result
                        .states
                        .push((CustomActionState::Move { position }, 1000));
                    return result;
                }
                CustomActionCreator::Sleep { .. } => {
                    result.states.push((CustomActionState::Sleep, 3000));
                    return result;
                }
                CustomActionCreator::Work { position, salary } => {
                    result
                        .creators
                        .push(CustomActionCreator::MoveTo { position });
                    result.creators.push(CustomActionCreator::Job { salary });
                    return result;
                }
                CustomActionCreator::EatFood { food_name } => {
                    result
                        .states
                        .push((CustomActionState::Eat { food_name }, 2000));
                    return result;
                }
                CustomActionCreator::Job { salary } => {
                    result
                        .states
                        .push((CustomActionState::Job { salary }, 5000));
                    return result;
                }
                _ => {}
            }

            JobResult {
                states: vec![(CustomActionState::Sleep, 1000)],
                creators: vec![],
            }
        }
    }

    #[test]
    fn test() {
        let entity_id_manager: IdManager<EntityId> = IdManager::new(0, |id| id + 1);
        let kind_components: ComponentGroup<Kind> = ComponentGroup::default();
        let name_components: ComponentGroup<String> = ComponentGroup::default();
        let position_components: ComponentGroup<Position> = ComponentGroup::default();

        let mut job_manager: JobManager<CustomGame> = JobManager::default();
        let mut custom_game = CustomGame {
            entity_id_manager,
            kind_components,
            name_components,
            position_components,
        };

        // Create cat
        {
            let entity_id = custom_game.entity_id_manager.create_generator().gen();
            custom_game.kind_components.push(entity_id, Kind::Cat);
            custom_game
                .name_components
                .push(entity_id, "Melon".to_string());
            custom_game
                .position_components
                .push(entity_id, Position((0, 0)));
            job_manager.action_controller().attach(entity_id);
        }
        // Create dog
        {
            let entity_id = custom_game.entity_id_manager.create_generator().gen();
            custom_game.kind_components.push(entity_id, Kind::Dog);
            custom_game
                .name_components
                .push(entity_id, "Puffball".to_string());
            custom_game
                .position_components
                .push(entity_id, Position((30, 10)));
            job_manager.action_controller().attach(entity_id);
        }
        // Create human
        {
            let entity_id = custom_game.entity_id_manager.create_generator().gen();
            custom_game.kind_components.push(entity_id, Kind::Human);
            custom_game
                .name_components
                .push(entity_id, "Jack".to_string());
            custom_game
                .position_components
                .push(entity_id, Position((4, 8)));
            job_manager.action_controller().attach(entity_id);
        }

        // Process horde
        job_manager.run(&mut custom_game, 50000);
    }
}
