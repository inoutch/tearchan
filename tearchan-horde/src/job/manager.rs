use crate::action::manager::{ActionManager, TimeMilliseconds};
use crate::action::result::ActionResult;
use crate::HordeInterface;
use std::collections::VecDeque;
use std::ops::Deref;
use std::option::Option::Some;
use tearchan_ecs::component::EntityId;

pub struct JobManager<T>
where
    T: HordeInterface,
{
    action_manager: ActionManager<T>,
    inner: T,
}

impl<T> JobManager<T>
where
    T: HordeInterface,
{
    pub fn new(provider: T) -> JobManager<T> {
        JobManager {
            action_manager: ActionManager::default(),
            inner: provider,
        }
    }

    pub fn run(&mut self, elapsed_time: TimeMilliseconds) {
        self.action_manager.update(elapsed_time);

        loop {
            self.update_action();

            let mut is_changed = false;
            let entity_ids = self.action_manager.clean_pending_entity_ids();
            for entity_id in entity_ids {
                let mut job_queue: VecDeque<T::Job> = VecDeque::new();
                job_queue.push_front(self.inner.on_first(entity_id));

                while let Some(job) = job_queue.pop_front() {
                    let result = self.inner.on_next(entity_id, job);
                    // Update actions
                    is_changed |= !result.states.is_empty();

                    // Add action creators
                    for creator in result.creators.into_iter().rev() {
                        job_queue.push_front(creator);
                    }

                    // Add actions
                    self.action_manager.push_states(entity_id, result.states);
                }
            }

            if !is_changed {
                break;
            }
        }
    }

    pub fn attach(&mut self, entity_id: EntityId) {
        self.action_manager.attach(entity_id);
    }

    pub fn detach(&mut self, entity_id: EntityId) {
        self.action_manager.detach(entity_id);
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> JobManager<T>
where
    T: HordeInterface,
{
    pub fn update_action(&mut self) {
        let mut results = self.action_manager.pull();
        while let Some(result) = results.pop_first_back() {
            match result {
                ActionResult::Start { action } => {
                    self.inner.on_start(action.deref());
                }
                ActionResult::Update {
                    action,
                    current_time,
                } => {
                    let duration = action.end_time() - action.start_time();
                    let ratio = (current_time - action.start_time()) as f32 / duration as f32;
                    self.inner.on_update(action.deref(), ratio);
                }
                ActionResult::End { action } => {
                    self.inner.on_end(action.deref());
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
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

        fn on_start(&mut self, _action: &Action<Self::ActionState>) {
            println!("start  : {:?}", _action);
        }

        fn on_update(&mut self, _action: &Action<Self::ActionState>, _ratio: f32) {
            println!("update : {:?}", _action);
        }

        fn on_end(&mut self, _action: &Action<Self::ActionState>) {
            println!("end    : {:?}", _action);
        }

        fn on_first(&self, entity_id: u32) -> Self::Job {
            let kind = self.kind_components.get(entity_id).unwrap();
            match kind {
                Kind::Dog => CustomActionCreator::EatLunch {
                    position: Position((100, 200)),
                    food_name: "dog food",
                },
                Kind::Cat => CustomActionCreator::Sleep,
                Kind::Human => CustomActionCreator::Work {
                    position: Position((100, 200)),
                    salary: 200, // $200
                },
            }
        }

        fn on_next(
            &self,
            _entity_id: u32,
            job: Self::Job,
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

        let mut action_creator_manager: JobManager<CustomGame> = JobManager::new(CustomGame {
            entity_id_manager,
            kind_components,
            name_components,
            position_components,
        });

        // Create cat
        {
            let entity_id = action_creator_manager
                .inner_mut()
                .entity_id_manager
                .create_generator()
                .gen();
            action_creator_manager
                .inner_mut()
                .kind_components
                .push(entity_id, Kind::Cat);
            action_creator_manager
                .inner_mut()
                .name_components
                .push(entity_id, "Melon".to_string());
            action_creator_manager
                .inner_mut()
                .position_components
                .push(entity_id, Position((0, 0)));
            action_creator_manager.attach(entity_id);
        }
        // Create dog
        {
            let entity_id = action_creator_manager
                .inner_mut()
                .entity_id_manager
                .create_generator()
                .gen();
            action_creator_manager
                .inner_mut()
                .kind_components
                .push(entity_id, Kind::Dog);
            action_creator_manager
                .inner_mut()
                .name_components
                .push(entity_id, "Puffball".to_string());
            action_creator_manager
                .inner_mut()
                .position_components
                .push(entity_id, Position((30, 10)));
            action_creator_manager.attach(entity_id);
        }
        // Create human
        {
            let entity_id = action_creator_manager
                .inner_mut()
                .entity_id_manager
                .create_generator()
                .gen();
            action_creator_manager
                .inner_mut()
                .kind_components
                .push(entity_id, Kind::Human);
            action_creator_manager
                .inner_mut()
                .name_components
                .push(entity_id, "Jack".to_string());
            action_creator_manager
                .inner_mut()
                .position_components
                .push(entity_id, Position((4, 8)));
            action_creator_manager.attach(entity_id);
        }

        // Process horde
        action_creator_manager.run(50000);
    }
}
