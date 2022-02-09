use crate::action::manager::{
    ActionClientManager, ActionController, ActionManager, ActionManagerData, ActionManagerTrait,
    ActionServerManager, TimeMilliseconds,
};
use crate::action::result::ActionResult;
use crate::HordeInterface;
use std::collections::VecDeque;
use std::ops::Deref;
use std::option::Option::Some;
use std::sync::Arc;
use tearchan_ecs::component::EntityId;

const MAX_LOOP_SIZE_PER_FRAME: usize = 1000usize;

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
        match &mut self.action_manager {
            ActionManager::Server(action_manager) => {
                action_manager.update(elapsed_time);

                let mut loop_count = 0;
                let mut next_action_time = action_manager.next_action_time();
                loop {
                    if next_action_time != action_manager.next_action_time() {
                        update_change_time(provider, action_manager);
                        next_action_time = action_manager.next_action_time();
                    }

                    let mut is_changed = update_action(provider, action_manager);
                    let entity_ids = action_manager.clean_pending_entity_ids();
                    for entity_id in entity_ids {
                        let mut priority = 0;
                        let mut job_queue: VecDeque<T::Job> = VecDeque::new();
                        job_queue.push_front(get_until_some_priority_is_returned(
                            |entity_id, priority| {
                                provider.on_first(entity_id, priority, &action_manager.reader())
                            },
                            entity_id,
                            &mut priority,
                        ));

                        while let Some(job) = job_queue.pop_front() {
                            let result = provider.on_next(entity_id, job, &action_manager.reader());
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
                                            &action_manager.reader(),
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
                            action_manager.push_states(entity_id, result.states);
                        }
                    }

                    if !is_changed {
                        break;
                    }
                    loop_count += 1;
                    debug_assert!(loop_count < MAX_LOOP_SIZE_PER_FRAME);
                }
            }
            ActionManager::Client(action_manager) => {
                action_manager.update(elapsed_time);
                update_action_in_client(provider, action_manager);
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

    pub fn replace_action_manager(&mut self, data: ActionManagerData<T::ActionState>) {
        self.action_manager = match &self.action_manager {
            ActionManager::Server(_) => ActionManager::Server(ActionServerManager::new(data)),
            ActionManager::Client(_) => ActionManager::Client(ActionClientManager::new(data)),
        };
    }
}

impl<T: HordeInterface> From<ActionManager<T::ActionState>> for JobManager<T> {
    fn from(action_manager: ActionManager<T::ActionState>) -> Self {
        JobManager { action_manager }
    }
}

fn update_action<T>(
    provider: &mut T,
    action_manager: &mut ActionServerManager<T::ActionState>,
) -> bool
where
    T: HordeInterface,
{
    let next_action_time = action_manager.next_action_time();
    while next_action_time == action_manager.next_action_time() {
        let result = match action_manager.pull() {
            None => return false,
            Some(result) => result,
        };
        let mut controller = action_manager.controller();
        update_event(provider, &mut controller, result);
    }
    true
}

fn update_change_time<T>(provider: &mut T, action_manager: &mut ActionServerManager<T::ActionState>)
where
    T: HordeInterface,
{
    for (entity_id, state) in provider
        .on_change_time(&action_manager.reader())
        .into_iter()
    {
        action_manager.push_states(entity_id, vec![state]);
    }
}

fn update_event<T>(
    provider: &mut T,
    controller: &mut ActionController<T::ActionState>,
    result: ActionResult<T::ActionState>,
) where
    T: HordeInterface,
{
    match result {
        ActionResult::Start { action } => {
            provider.on_send(Arc::clone(&action));
            provider.on_start(action.deref(), controller);
        }
        ActionResult::Update {
            action,
            current_time,
        } => {
            let duration = action.end_time() - action.start_time();
            let ratio = (current_time - action.start_time()) as f32 / duration as f32;
            provider.on_update(action.deref(), ratio, controller);
        }
        ActionResult::End { action } => {
            provider.on_end(action.deref(), controller);
        }
        ActionResult::Cancel { action } => {
            provider.on_cancel(&action, controller);
        }
        ActionResult::Enqueue { action } => {
            provider.on_enqueue(&action, controller);
        }
    }
}

fn update_action_in_client<T, U>(provider: &mut T, action_manager: &mut U)
where
    T: HordeInterface,
    U: ActionManagerTrait<T::ActionState>,
{
    while let Some(result) = action_manager.pull() {
        let mut controller = action_manager.controller();
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
            ActionResult::Cancel { action } => {
                provider.on_cancel(&action, &mut controller);
            }
            ActionResult::Enqueue { action } => {
                provider.on_enqueue(&action, &mut controller);
            }
        }
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
    use crate::action::manager::{ActionController, ActionServerReader, TimeMilliseconds};
    use crate::action::Action;
    use crate::job::manager::JobManager;
    use crate::job::result::JobResult;
    use crate::{HordeInterface, ProgressState};
    use std::cell::RefCell;
    use tearchan_ecs::component::group::ComponentGroup;
    use tearchan_ecs::component::EntityId;
    use tearchan_util::id_manager::IdManager;

    #[derive(Debug)]
    enum Kind {
        Dog,
        Cat,
        Human,
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    enum CustomActionState {
        Move { position: Position },
        Sleep,
        Eat { food_name: &'static str },
        Job { salary: u32 },
    }

    #[derive(Debug, Clone)]
    struct Position((u32, u32));

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
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
        Invalid,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    enum ActionKind {
        Start {
            current_time: TimeMilliseconds,
            action: Action<CustomActionState>,
        },
        End {
            current_time: TimeMilliseconds,
            action: Action<CustomActionState>,
        },
        Update {
            current_time: TimeMilliseconds,
            action: Action<CustomActionState>,
            ratio: f32,
        },
        Enqueue {
            action: Action<CustomActionState>,
        },
        First {
            entity_id: EntityId,
            priority: u32,
            current_time: TimeMilliseconds,
        },
        Next {
            entity_id: EntityId,
            job: CustomActionCreator,
            current_time: TimeMilliseconds,
        },
        Cancel {
            action: Action<CustomActionState>,
        },
        ChangeTime {
            current_time: TimeMilliseconds,
        },
    }

    struct CustomGame {
        pub entity_id_manager: IdManager<EntityId>,
        pub kind_components: ComponentGroup<Kind>,
        pub name_components: ComponentGroup<String>,
        pub position_components: ComponentGroup<Position>,
        pub actions: RefCell<Vec<ActionKind>>,
    }

    impl HordeInterface for CustomGame {
        type ActionState = CustomActionState;
        type Job = CustomActionCreator;

        fn on_start(
            &mut self,
            action: &Action<Self::ActionState>,
            _controller: &mut ActionController<CustomActionState>,
        ) {
            self.actions.borrow_mut().push(ActionKind::Start {
                current_time: action.start_time(),
                action: action.clone(),
            });
        }

        fn on_update(
            &mut self,
            action: &Action<Self::ActionState>,
            ratio: f32,
            _controller: &mut ActionController<CustomActionState>,
        ) {
            self.actions.borrow_mut().push(ActionKind::Update {
                current_time: (((action.end_time() - action.start_time()) as f32 * ratio)
                    as TimeMilliseconds
                    + action.start_time()),
                action: action.clone(),
                ratio,
            });
        }

        fn on_end(
            &mut self,
            action: &Action<Self::ActionState>,
            _controller: &mut ActionController<CustomActionState>,
        ) {
            self.actions.borrow_mut().push(ActionKind::End {
                current_time: action.end_time(),
                action: action.clone(),
            });
        }

        fn on_enqueue(
            &mut self,
            action: &Action<Self::ActionState>,
            _controller: &mut ActionController<CustomActionState>,
        ) {
            self.actions.borrow_mut().push(ActionKind::Enqueue {
                action: action.clone(),
            });
        }

        fn on_first(
            &self,
            entity_id: EntityId,
            priority: u32,
            reader: &ActionServerReader<Self::ActionState>,
        ) -> Option<Self::Job> {
            self.actions.borrow_mut().push(ActionKind::First {
                entity_id,
                priority,
                current_time: reader.current_time(),
            });
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
            entity_id: EntityId,
            job: Self::Job,
            reader: &ActionServerReader<Self::ActionState>,
        ) -> JobResult<Self::Job, Self::ActionState> {
            self.actions.borrow_mut().push(ActionKind::Next {
                entity_id,
                job: job.clone(),
                current_time: reader.current_time(),
            });
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

        fn on_cancel(
            &mut self,
            action: &Action<Self::ActionState>,
            _controller: &mut ActionController<Self::ActionState>,
        ) {
            self.actions.borrow_mut().push(ActionKind::Cancel {
                action: action.clone(),
            });
        }

        fn on_change_time(
            &mut self,
            reader: &ActionServerReader<Self::ActionState>,
        ) -> Vec<(EntityId, ProgressState<Self::ActionState>)> {
            self.actions.borrow_mut().push(ActionKind::ChangeTime {
                current_time: reader.current_time(),
            });
            Vec::with_capacity(0)
        }
    }

    #[test]
    fn test_custom_game() {
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
            actions: RefCell::new(Vec::new()),
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
        job_manager.run(&mut custom_game, 10000);

        insta::assert_debug_snapshot!("actions", custom_game.actions.borrow());
    }
}
