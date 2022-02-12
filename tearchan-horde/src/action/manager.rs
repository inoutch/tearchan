use crate::action::context::ActionContext;
use crate::action::result::ActionResult;
use crate::action::Action;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tearchan_ecs::component::EntityId;
use tearchan_util::btree::DuplicatableBTreeMap;

pub type TimeMilliseconds = u64;

#[derive(Debug)]
struct CommandState<T> {
    action: Arc<Action<T>>,
    is_active: AtomicBool,
}

impl<T> CommandState<T> {
    pub fn new(action: Arc<Action<T>>) -> CommandState<T> {
        Self {
            action,
            is_active: AtomicBool::new(true),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }

    pub fn inactive(&self) {
        self.is_active.store(false, Ordering::Relaxed);
    }
}

#[derive(Debug)]
enum Command<T> {
    Start {
        state: Arc<CommandState<T>>,
        time: TimeMilliseconds,
    },
    Update {
        state: Arc<CommandState<T>>,
        time: TimeMilliseconds,
    },
    End {
        state: Arc<CommandState<T>>,
        time: TimeMilliseconds,
    },
    Cancel {
        state: Arc<CommandState<T>>,
        time: TimeMilliseconds,
    },
    Enqueue {
        state: Arc<CommandState<T>>,
        time: TimeMilliseconds,
    },
}

impl<T> Command<T> {
    pub fn state(&self) -> &CommandState<T> {
        match self {
            Command::Start { state, .. } => state,
            Command::Update { state, .. } => state,
            Command::End { state, .. } => state,
            Command::Cancel { state, .. } => state,
            Command::Enqueue { state, .. } => state,
        }
    }

    pub fn time(&self) -> TimeMilliseconds {
        match self {
            Command::Start { time, .. } => *time,
            Command::Update { time, .. } => *time,
            Command::End { time, .. } => *time,
            Command::Cancel { time, .. } => *time,
            Command::Enqueue { time, .. } => *time,
        }
    }
}

pub enum ActionManager<T> {
    Server(ActionServerManager<T>),
    Client(ActionClientManager<T>),
}

impl<T> Default for ActionManager<T> {
    fn default() -> Self {
        ActionManager::Server(ActionServerManager::default())
    }
}

impl<T> ActionManager<T> {
    pub fn new_server(data: ActionManagerData<T>) -> Self {
        ActionManager::Server(ActionServerManager::new(data))
    }

    pub fn new_client(data: ActionManagerData<T>) -> Self {
        ActionManager::Client(ActionClientManager::new(data))
    }

    pub fn controller(&mut self) -> ActionController<T> {
        match self {
            ActionManager::Server(manager) => manager.controller(),
            ActionManager::Client(manager) => manager.controller(),
        }
    }

    pub fn current_time(&self) -> TimeMilliseconds {
        match self {
            ActionManager::Server(manager) => manager.current_time,
            ActionManager::Client(manager) => manager.current_time,
        }
    }

    pub fn create_data(&self) -> ActionManagerData<T> {
        match self {
            ActionManager::Server(manager) => manager.create_data(),
            ActionManager::Client(_) => panic!("Unsupported"),
        }
    }

    pub fn get_running_action(&self, entity_id: EntityId) -> Option<&Arc<Action<T>>> {
        match self {
            ActionManager::Server(manager) => manager.get_running_action(entity_id),
            ActionManager::Client(manager) => manager.get_running_action(entity_id),
        }
    }
}

pub trait ActionManagerTrait<T> {
    fn pull(&mut self) -> Option<ActionResult<T>>;

    fn controller(&mut self) -> ActionController<T>;

    fn get_running_action(&self, entity_id: EntityId) -> Option<&Arc<Action<T>>>;
}

pub struct ActionServerManager<T> {
    // Do serialize
    commands: DuplicatableBTreeMap<TimeMilliseconds, Command<T>>,
    actions: HashMap<EntityId, DuplicatableBTreeMap<TimeMilliseconds, Arc<CommandState<T>>>>,
    running_actions: BTreeMap<EntityId, Arc<Action<T>>>,
    current_time: TimeMilliseconds,
    next_time: TimeMilliseconds,
    // Cache and using for server only
    contexts: HashMap<EntityId, ActionContext>,
    pending_cache: BTreeSet<EntityId>,
}

impl<T> Default for ActionServerManager<T> {
    fn default() -> Self {
        ActionServerManager {
            commands: DuplicatableBTreeMap::default(),
            actions: HashMap::new(),
            running_actions: BTreeMap::new(),
            current_time: 0,
            next_time: 0,
            contexts: HashMap::new(),
            pending_cache: BTreeSet::new(),
        }
    }
}

impl<T> ActionServerManager<T> {
    pub fn new(data: ActionManagerData<T>) -> ActionServerManager<T> {
        let mut commands = DuplicatableBTreeMap::default();
        let mut actions = HashMap::new();
        let mut running_actions = BTreeMap::new();
        let mut contexts = HashMap::default();

        for action in data.actions {
            let mut context =
                get_or_create_context_mut(action.entity_id, data.current_time, &mut contexts);

            let command_state = Arc::new(CommandState::new(Arc::clone(&action)));
            if data.current_time <= action.start_time {
                commands.push_back(
                    action.start_time,
                    Command::Start {
                        state: Arc::clone(&command_state),
                        time: action.start_time,
                    },
                );
            } else {
                context.last_time = action.end_time;
                running_actions.insert(action.entity_id, Arc::clone(&action));
            }

            commands.push_back(
                action.end_time,
                Command::End {
                    state: Arc::clone(&command_state),
                    time: action.end_time,
                },
            );

            actions
                .entry(action.entity_id)
                .or_insert_with(DuplicatableBTreeMap::default)
                .push_back(action.start_time, command_state);
        }

        ActionServerManager {
            commands,
            actions,
            running_actions,
            current_time: data.current_time,
            next_time: data.current_time,
            contexts,
            pending_cache: BTreeSet::new(),
        }
    }

    pub fn update(&mut self, elapsed_time: TimeMilliseconds) {
        update(
            &mut self.commands,
            &mut self.running_actions,
            &mut self.next_time,
            elapsed_time,
        );
    }

    pub fn is_running(&self, entity_id: EntityId) -> bool {
        match self.actions.get(&entity_id) {
            Some(actions) => !actions.is_empty(),
            None => false,
        }
    }

    pub fn push_states(&mut self, entity_id: EntityId, states: Vec<(T, TimeMilliseconds)>) {
        let mut actions = Vec::new();
        let mut last_time =
            get_context_mut(entity_id, self.current_time, &mut self.contexts).last_time;
        for (state, duration) in states {
            let end_time = last_time.saturating_add(duration);
            actions.push(Arc::new(Action::new(entity_id, last_time, end_time, state)));
            last_time = end_time;
        }

        for action in actions.iter() {
            let action = Arc::clone(action);
            self.commands.push_back(
                self.current_time,
                Command::Enqueue {
                    state: Arc::new(CommandState::new(Arc::clone(&action))),
                    time: self.current_time,
                },
            );
        }

        for action in actions.iter() {
            let start_time = action.start_time;
            let end_time = action.end_time;
            let action = Arc::clone(action);
            let command_state = Arc::new(CommandState::new(Arc::clone(&action)));

            self.commands.push_back(
                start_time,
                Command::Start {
                    state: Arc::clone(&command_state),
                    time: start_time,
                },
            );
            self.commands.push_back(
                end_time,
                Command::End {
                    state: Arc::clone(&command_state),
                    time: end_time,
                },
            );
            self.actions
                .get_mut(&action.entity_id)
                .unwrap_or_else(|| panic!("{} of entity is not attached", action.entity_id))
                .push_back(action.start_time, command_state);

            let context = get_context_mut(entity_id, self.current_time, &mut self.contexts);
            context.last_time = end_time;
        }
    }

    /**
     * Get free entity ids after update_actions
     */
    pub fn clean_pending_entity_ids(&mut self) -> BTreeSet<EntityId> {
        std::mem::take(&mut self.pending_cache)
    }

    pub fn attach(&mut self, entity_id: EntityId) {
        debug_assert!(
            !self.contexts.contains_key(&entity_id),
            "entity of {} is already attached",
            entity_id
        );

        let context = ActionContext {
            last_time: self.current_time,
            running_end_time: self.current_time,
        };
        self.contexts.insert(entity_id, context);
        self.pending_cache.insert(entity_id);
        self.actions
            .insert(entity_id, DuplicatableBTreeMap::default());
    }

    pub fn detach(&mut self, entity_id: EntityId) {
        self.contexts.remove(&entity_id);
        self.pending_cache.remove(&entity_id);
        self.running_actions.remove(&entity_id);
        if let Some(mut states) = self.actions.remove(&entity_id) {
            while let Some(state) = states.pop_first_back() {
                state.inactive();
            }
        }
    }

    pub fn cancel(&mut self, entity_id: EntityId, immediate: bool) {
        let current_time = self.current_time;
        let context = get_context_mut(entity_id, self.current_time, &mut self.contexts);
        let mut commands = Vec::new();

        if immediate {
            context.last_time = current_time;

            self.pending_cache.insert(entity_id);
            self.running_actions.remove(&entity_id);
            let actions = self.actions.get_mut(&entity_id).unwrap();
            while let Some(command_state) = actions.pop_first_back() {
                match current_time.cmp(&command_state.action.end_time) {
                    std::cmp::Ordering::Less => {
                        command_state.inactive();
                        commands.push(Command::Cancel {
                            state: Arc::new(CommandState::new(Arc::clone(&command_state.action))),
                            time: current_time,
                        });
                    }
                    std::cmp::Ordering::Equal => {
                        if current_time == command_state.action.start_time {
                            // If the timing of the cancellation is the same as the action to be executed,
                            // only the actions that cannot be executed immediately will be inactive.
                            command_state.inactive();

                            let command_state =
                                Arc::new(CommandState::new(Arc::clone(&command_state.action)));
                            commands.push(Command::Start {
                                state: Arc::clone(&command_state),
                                time: current_time,
                            });
                            commands.push(Command::Cancel {
                                state: command_state,
                                time: current_time,
                            });
                        } else {
                            command_state.inactive();
                            let command_state =
                                Arc::new(CommandState::new(Arc::clone(&command_state.action)));
                            commands.push(Command::Cancel {
                                state: command_state,
                                time: current_time,
                            });
                        }
                    }
                    std::cmp::Ordering::Greater => {}
                }
            }
        } else {
            // Find a running action of entity_id
            let actions = self.actions.get_mut(&entity_id).unwrap();
            context.last_time = self
                .running_actions
                .get(&entity_id)
                .map(|action| action.end_time)
                .unwrap_or(self.current_time);

            let mut running_actions = Vec::new();
            while let Some(command_state) = actions.pop_first_back() {
                if current_time < command_state.action.start_time {
                    command_state.inactive();
                    commands.push(Command::Cancel {
                        state: Arc::new(CommandState::new(Arc::clone(&command_state.action))),
                        time: current_time,
                    });
                } else if current_time == command_state.action.start_time
                    && current_time != command_state.action.end_time
                {
                    // If the timing of the cancellation is the same as the action to be executed,
                    // only the actions that cannot be executed immediately will be inactive.
                    command_state.inactive();

                    let command_state =
                        Arc::new(CommandState::new(Arc::clone(&command_state.action)));
                    commands.push(Command::Start {
                        state: Arc::clone(&command_state),
                        time: current_time,
                    });
                    commands.push(Command::Cancel {
                        state: command_state,
                        time: current_time,
                    });
                } else {
                    running_actions.push(command_state);
                }
            }

            for running_action in running_actions {
                actions.push_back(running_action.action.start_time, running_action);
            }
        }

        for command in commands {
            self.commands.push_back(self.current_time, command);
        }
    }

    pub fn reader(&self) -> ActionServerReader<T> {
        ActionServerReader {
            action_manager: self,
        }
    }

    pub fn current_time(&self) -> TimeMilliseconds {
        self.current_time
    }

    pub fn next_action_time(&self) -> TimeMilliseconds {
        self.commands
            .iter()
            .find_map(|(key, values)| {
                if values
                    .iter()
                    .find(|command| command.state().is_active())
                    .is_some()
                {
                    Some(*key)
                } else {
                    None
                }
            })
            .unwrap_or(self.current_time)
    }

    pub fn create_data(&self) -> ActionManagerData<T> {
        let mut actions = Vec::new();
        for (_, commands) in self.commands.iter() {
            for command in commands {
                if let Command::End { state, .. } = command {
                    if state.is_active() {
                        actions.push(Arc::clone(&state.action));
                    }
                }
            }
        }

        ActionManagerData {
            actions,
            entity_ids: self
                .contexts
                .iter()
                .map(|(entity_id, _)| *entity_id)
                .collect(),
            current_time: self.current_time,
        }
    }
}

impl<T> ActionManagerTrait<T> for ActionServerManager<T> {
    fn pull(&mut self) -> Option<ActionResult<T>> {
        while let Some(command) = self.commands.pop_first_back() {
            if !command.state().is_active() {
                continue;
            }

            if self.next_time < command.time() {
                self.current_time = self.next_time;
                self.commands.push_front(command.time(), command);
                break;
            }

            self.current_time = command.time();

            return match command {
                Command::Start { state, .. } => {
                    let context = get_context_mut(
                        state.action.entity_id,
                        self.current_time,
                        &mut self.contexts,
                    );
                    context.last_time = state.action.end_time;

                    self.running_actions
                        .insert(state.action.entity_id, Arc::clone(&state.action));
                    if state.action.start_time < self.next_time
                        && self.next_time < state.action.end_time
                    {
                        self.commands.push_back(
                            self.next_time,
                            Command::Update {
                                state: Arc::new(CommandState::new(Arc::clone(&state.action))),
                                time: self.next_time,
                            },
                        );
                    }

                    Some(ActionResult::Start {
                        action: Arc::clone(&state.action),
                    })
                }
                Command::Update { state, time } => Some(ActionResult::Update {
                    action: Arc::clone(&state.action),
                    current_time: time,
                }),
                Command::End { state, .. } => {
                    let entity_id = state.action.entity_id;
                    pop_until_active_action(&mut self.actions, entity_id);
                    if self.actions.get(&entity_id).unwrap().is_empty() {
                        self.pending_cache.insert(entity_id);
                    }

                    Some(ActionResult::End {
                        action: Arc::clone(&state.action),
                    })
                }
                Command::Cancel { state, .. } => Some(ActionResult::Cancel {
                    action: Arc::clone(&state.action),
                }),
                Command::Enqueue { state, .. } => Some(ActionResult::Enqueue {
                    action: Arc::clone(&state.action),
                }),
            };
        }
        None
    }

    fn controller(&mut self) -> ActionController<T> {
        ActionController {
            action_manager: ActionManagerRef::Server(self),
        }
    }

    fn get_running_action(&self, entity_id: EntityId) -> Option<&Arc<Action<T>>> {
        self.running_actions.get(&entity_id)
    }
}

pub struct ActionClientManager<T> {
    // Do serialize
    commands: DuplicatableBTreeMap<TimeMilliseconds, Command<T>>,
    last_actions: HashMap<EntityId, Arc<CommandState<T>>>,
    running_actions: BTreeMap<EntityId, Arc<Action<T>>>,
    current_time: TimeMilliseconds,
    next_time: TimeMilliseconds,
}

impl<T> ActionClientManager<T> {
    pub fn new(data: ActionManagerData<T>) -> Self {
        let mut commands = DuplicatableBTreeMap::default();
        let mut last_actions = HashMap::new();
        let mut running_actions = BTreeMap::new();

        for action in data.actions {
            let command_state = Arc::new(CommandState::new(Arc::clone(&action)));
            if data.current_time <= action.start_time {
                commands.push_back(
                    action.start_time,
                    Command::Start {
                        state: Arc::clone(&command_state),
                        time: action.start_time,
                    },
                );
            } else {
                running_actions.insert(action.entity_id(), Arc::clone(&action));
            }

            commands.push_back(
                action.end_time,
                Command::End {
                    state: Arc::new(CommandState::new(Arc::clone(&action))),
                    time: action.end_time,
                },
            );

            let last_action = last_actions
                .entry(action.entity_id)
                .or_insert_with(|| Arc::clone(&command_state));
            if last_action.action.start_time < action.start_time {
                *last_action = command_state;
            }
        }

        ActionClientManager {
            commands,
            last_actions,
            running_actions,
            current_time: data.current_time,
            next_time: data.current_time,
        }
    }

    pub fn update(&mut self, elapsed_time: TimeMilliseconds) {
        update(
            &mut self.commands,
            &mut self.running_actions,
            &mut self.next_time,
            elapsed_time,
        );
    }

    pub fn push_actions(&mut self, actions: Vec<Arc<Action<T>>>) {
        for action in actions {
            let command_state = Arc::new(CommandState::new(Arc::clone(&action)));

            if let Some(last_command_state) = self.last_actions.get(&action.entity_id) {
                let last_time = last_command_state.action.end_time;
                if action.start_time < last_time {
                    self.commands.push_back(
                        action.start_time,
                        Command::Cancel {
                            state: Arc::clone(last_command_state),
                            time: action.start_time,
                        },
                    );
                }
            }

            self.commands.push_back(
                action.start_time,
                Command::Start {
                    state: Arc::clone(&command_state),
                    time: action.start_time,
                },
            );
            self.commands.push_back(
                action.end_time,
                Command::End {
                    state: Arc::clone(&command_state),
                    time: action.end_time,
                },
            );

            let last_command_state = self
                .last_actions
                .entry(action.entity_id)
                .or_insert_with(|| Arc::clone(&command_state));
            if last_command_state.action.start_time < action.start_time {
                *last_command_state = command_state;
            }
        }
    }
}

impl<T> ActionManagerTrait<T> for ActionClientManager<T> {
    fn pull(&mut self) -> Option<ActionResult<T>> {
        while let Some(command) = self.commands.pop_first_back() {
            if !command.state().is_active() {
                continue;
            }

            if self.next_time < command.time() {
                self.current_time = self.next_time;
                self.commands.push_front(command.time(), command);
                break;
            }

            self.current_time = command.time();

            return match command {
                Command::Start { state, .. } => {
                    self.running_actions
                        .insert(state.action.entity_id, Arc::clone(&state.action));
                    if state.action.start_time < self.next_time
                        && self.next_time < state.action.end_time
                    {
                        self.commands.push_back(
                            self.next_time,
                            Command::Update {
                                state: Arc::new(CommandState::new(Arc::clone(&state.action))),
                                time: self.next_time,
                            },
                        );
                    }

                    Some(ActionResult::Start {
                        action: Arc::clone(&state.action),
                    })
                }
                Command::Update { state, time } => Some(ActionResult::Update {
                    action: Arc::clone(&state.action),
                    current_time: time,
                }),
                Command::End { state, .. } => {
                    let entity_id = state.action.entity_id;
                    if let Some(command_state) = self.last_actions.get(&entity_id) {
                        if command_state.action.start_time == state.action.start_time {
                            self.last_actions.remove(&entity_id);
                        }
                    }

                    self.running_actions.remove(&entity_id);

                    Some(ActionResult::End {
                        action: Arc::clone(&state.action),
                    })
                }
                Command::Cancel { state, .. } => {
                    state.inactive();
                    Some(ActionResult::Cancel {
                        action: Arc::clone(&state.action),
                    })
                }
                Command::Enqueue { .. } => continue,
            };
        }
        None
    }

    fn controller(&mut self) -> ActionController<T> {
        ActionController {
            action_manager: ActionManagerRef::Client(self),
        }
    }

    fn get_running_action(&self, entity_id: EntityId) -> Option<&Arc<Action<T>>> {
        self.running_actions.get(&entity_id)
    }
}

#[inline]
fn update<T>(
    commands: &mut DuplicatableBTreeMap<TimeMilliseconds, Command<T>>,
    running_actions: &mut BTreeMap<EntityId, Arc<Action<T>>>,
    next_time: &mut TimeMilliseconds,
    elapsed_time: TimeMilliseconds,
) {
    *next_time += elapsed_time;

    for (_, action) in running_actions.iter() {
        if *next_time < action.end_time {
            commands.push_back(
                *next_time,
                Command::Update {
                    state: Arc::new(CommandState::new(Arc::clone(action))),
                    time: *next_time,
                },
            );
        }
    }
}

#[inline]
fn get_context_mut(
    entity_id: EntityId,
    current_time: TimeMilliseconds,
    contexts: &mut HashMap<EntityId, ActionContext>,
) -> &mut ActionContext {
    debug_assert!(
        contexts.contains_key(&entity_id),
        "entity of {} is not attached",
        entity_id
    );

    get_or_create_context_mut(entity_id, current_time, contexts)
}

#[inline]
fn get_or_create_context_mut(
    entity_id: EntityId,
    current_time: TimeMilliseconds,
    contexts: &mut HashMap<EntityId, ActionContext>,
) -> &mut ActionContext {
    if contexts.get_mut(&entity_id).is_some() {
        return contexts.get_mut(&entity_id).unwrap();
    }

    let context = ActionContext {
        last_time: current_time,
        running_end_time: current_time,
    };
    contexts.insert(entity_id, context);
    contexts.get_mut(&entity_id).unwrap()
}

fn pop_until_active_action<T>(
    actions: &mut HashMap<EntityId, DuplicatableBTreeMap<TimeMilliseconds, Arc<CommandState<T>>>>,
    entity_id: EntityId,
) {
    let actions = actions
        .get_mut(&entity_id)
        .unwrap_or_else(|| panic!("{} of entity is not attached", entity_id));
    while let Some(action) = actions.pop_first_back() {
        if action.is_active() {
            break;
        }
    }
}

enum ActionManagerRef<'a, T> {
    Server(&'a mut ActionServerManager<T>),
    Client(&'a mut ActionClientManager<T>),
}

pub struct ActionController<'a, T> {
    action_manager: ActionManagerRef<'a, T>,
}

impl<'a, T> ActionController<'a, T> {
    #[inline]
    pub fn attach(&mut self, entity_id: EntityId) {
        match &mut self.action_manager {
            ActionManagerRef::Server(server) => server.attach(entity_id),
            ActionManagerRef::Client(_) => {}
        };
    }

    #[inline]
    pub fn detach(&mut self, entity_id: EntityId) {
        match &mut self.action_manager {
            ActionManagerRef::Server(server) => server.detach(entity_id),
            ActionManagerRef::Client(_) => {}
        };
    }

    #[inline]
    pub fn cancel(&mut self, entity_id: EntityId, immediate: bool) {
        match &mut self.action_manager {
            ActionManagerRef::Server(server) => server.cancel(entity_id, immediate),
            ActionManagerRef::Client(_) => {}
        };
    }
}

pub struct ActionServerReader<'a, T> {
    action_manager: &'a ActionServerManager<T>,
}

impl<'a, T> ActionServerReader<'a, T> {
    /// The time for the next action to be processed
    #[inline]
    pub fn current_time(&self) -> TimeMilliseconds {
        self.action_manager.current_time
    }

    /// The time of the entity's last action
    /// When generating a job, it means the start time of the next action
    #[inline]
    pub fn last_time(&self, entity_id: EntityId) -> Option<TimeMilliseconds> {
        self.action_manager
            .contexts
            .get(&entity_id)
            .map(|context| context.last_time)
    }
}

pub struct ActionClientReader<'a, T> {
    action_manager: &'a ActionClientManager<T>,
}

impl<'a, T> ActionClientReader<'a, T> {
    /// The time for the next action to be processed
    #[inline]
    pub fn current_time(&self) -> TimeMilliseconds {
        self.action_manager.current_time
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActionManagerData<T> {
    pub actions: Vec<Arc<Action<T>>>,
    #[serde(rename = "entityIds")]
    pub entity_ids: HashSet<EntityId>,
    #[serde(rename = "currentTime")]
    pub current_time: TimeMilliseconds,
}

impl<T> Default for ActionManagerData<T> {
    fn default() -> Self {
        ActionManagerData {
            actions: Vec::new(),
            entity_ids: HashSet::new(),
            current_time: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::action::manager::{
        ActionClientManager, ActionManagerData, ActionManagerTrait, ActionServerManager,
    };
    use crate::action::result::ActionResult;
    use crate::action::Action;
    use std::sync::Arc;

    type TestActionState = &'static str;

    fn asset_action(actual: &Action<TestActionState>, expect: &Action<TestActionState>) {
        assert_eq!(
            actual.entity_id, expect.entity_id,
            "entity_id is not correct\n   actual: {:?}\n   expect: {:?}",
            actual, expect
        );
        assert_eq!(
            actual.inner, expect.inner,
            "inner is not correct\n   actual: {:?}\n   expect: {:?}",
            actual, expect
        );
        assert_eq!(
            actual.start_time, expect.start_time,
            "start_time is not correct\n   actual: {:?}\n   expect: {:?}",
            actual, expect
        );
        assert_eq!(
            actual.end_time, expect.end_time,
            "end_time is not correct\n   actual: {:?}\n   expect: {:?}",
            actual, expect
        );
    }

    fn asset_action_result(
        actual: Option<ActionResult<TestActionState>>,
        expect: Option<ActionResult<TestActionState>>,
    ) {
        if expect.is_none() {
            assert_eq!(
                actual.is_none(),
                expect.is_none(),
                "{:?} is not None",
                actual
            );
            return;
        }

        let actual = actual.unwrap_or_else(|| panic!("actual is None"));
        let expect = expect.unwrap();

        match expect {
            ActionResult::Start { action, .. } => {
                let actual = actual
                    .get_start()
                    .unwrap_or_else(|| panic!("{:?} is not start", actual));
                asset_action(actual, &action);
            }
            ActionResult::Update {
                action,
                current_time,
            } => {
                if let ActionResult::Update {
                    current_time: actual_current_time,
                    ..
                } = &actual
                {
                    assert_eq!(
                        *actual_current_time, current_time,
                        "current_time is not correct\n   actual: {:?}\n   expect: {:?}",
                        *actual_current_time, current_time
                    )
                }
                let (actual, _) = actual
                    .get_update()
                    .unwrap_or_else(|| panic!("{:?} is not update", actual));
                asset_action(actual, &action);
            }
            ActionResult::End { action, .. } => {
                let actual = actual
                    .get_end()
                    .unwrap_or_else(|| panic!("{:?} is not end", actual));
                asset_action(actual, &action);
            }
            ActionResult::Cancel { action, .. } => {
                let actual = actual
                    .get_cancel()
                    .unwrap_or_else(|| panic!("{:?} is not cancel", actual));
                asset_action(actual, &action);
            }
            ActionResult::Enqueue { action, .. } => {
                let actual = actual
                    .get_enqueue()
                    .unwrap_or_else(|| panic!("{:?} is not enqueue", actual));
                asset_action(actual, &action);
            }
        }
    }

    #[test]
    fn test_multiple_entities() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);
        action_manager.attach(2);

        action_manager.push_states(1, vec![("Sleep", 1000), ("Walk", 3000), ("Smiling", 1000)]);
        action_manager.push_states(2, vec![("Sleep", 2000), ("Walk", 2000), ("Eat", 1000)]);

        action_manager.update(500);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 1000, 4000, "Walk")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 4000, 5000, "Smiling")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep")),
            }),
        );

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(2, 0, 2000, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(2, 2000, 4000, "Walk")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(2, 4000, 5000, "Eat")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(2, 0, 2000, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Update {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep")),
                current_time: 500,
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Update {
                action: Arc::new(Action::new(2, 0, 2000, "Sleep")),
                current_time: 500,
            }),
        );
        asset_action_result(action_manager.pull(), None);
    }

    #[test]
    fn test_multiple_cancel() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);
        action_manager.push_states(1, vec![("Sleep", 2000), ("Walk", 3000)]);
        action_manager.attach(2);
        action_manager.push_states(2, vec![("Sleep", 3000), ("Walk", 2000)]);

        assert_eq!(action_manager.contexts.get(&1).unwrap().last_time, 5000);

        action_manager.update(1000);

        while action_manager.pull().is_some() {}

        action_manager.cancel(1, false);
        assert_eq!(action_manager.contexts.get(&1).unwrap().last_time, 2000);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Cancel {
                action: Arc::new(Action::new(1, 2000, 5000, "Walk")),
            }),
        );

        asset_action_result(action_manager.pull(), None);

        action_manager.update(1000);

        action_manager.push_states(1, vec![("Eat", 5000)]);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 2000, 7000, "Eat")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 0, 2000, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Update {
                action: Arc::new(Action::new(2, 0, 3000, "Sleep")),
                current_time: 2000,
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 2000, 7000, "Eat")),
            }),
        );
        asset_action_result(action_manager.pull(), None);
    }

    #[test]
    fn test_not_update() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);
        action_manager.push_states(1, vec![("Sleep", 1000)]);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep")),
            }),
        );
        asset_action_result(action_manager.pull(), None);
    }

    #[test]
    #[should_panic(expected = "entity of 1 is not attached")]
    fn test_not_attach() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.push_states(1, vec![("Sleep", 1000)]);
    }

    #[test]
    fn test_single_cancel() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);
        action_manager.push_states(1, vec![("Sleep", 2000), ("Walk", 3000), ("Eat", 1000)]);

        assert_eq!(action_manager.contexts.get(&1).unwrap().last_time, 6000);

        action_manager.update(2500);

        while action_manager.pull().is_some() {}

        action_manager.cancel(1, false);
        assert_eq!(action_manager.contexts.get(&1).unwrap().last_time, 5000);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Cancel {
                action: Arc::new(Action::new(1, 5000, 6000, "Eat")),
            }),
        );

        asset_action_result(action_manager.pull(), None);

        action_manager.update(3500);
        assert_eq!(action_manager.next_time, 6000);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 2000, 5000, "Walk")),
            }),
        );

        action_manager.push_states(1, vec![("Eat", 5000)]);
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 5000, 10000, "Eat")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 5000, 10000, "Eat")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Update {
                action: Arc::new(Action::new(1, 5000, 10000, "Eat")),
                current_time: 6000,
            }),
        );
        asset_action_result(action_manager.pull(), None);
    }

    #[test]
    fn test_single_cancel_immediately() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);
        action_manager.push_states(1, vec![("Sleep", 2000), ("Walk", 3000)]);

        assert_eq!(action_manager.contexts.get(&1).unwrap().last_time, 5000);

        action_manager.update(1000);

        while action_manager.pull().is_some() {}

        action_manager.cancel(1, true);
        assert_eq!(action_manager.contexts.get(&1).unwrap().last_time, 1000);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Cancel {
                action: Arc::new(Action::new(1, 0, 2000, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Cancel {
                action: Arc::new(Action::new(1, 2000, 5000, "Walk")),
            }),
        );

        asset_action_result(action_manager.pull(), None);

        action_manager.push_states(1, vec![("Eat", 5000)]);

        action_manager.update(1000);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 1000, 6000, "Eat")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 1000, 6000, "Eat")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Update {
                action: Arc::new(Action::new(1, 1000, 6000, "Eat")),
                current_time: 2000,
            }),
        );
        asset_action_result(action_manager.pull(), None);
    }

    #[test]
    fn test_single_clean_pending_entity_ids() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);

        let ids = action_manager.clean_pending_entity_ids();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&1));

        action_manager.push_states(1, vec![("Sleep", 2000), ("Walk", 3000)]);

        let ids = action_manager.clean_pending_entity_ids();
        assert_eq!(ids.len(), 0);

        action_manager.update(2500);
        while action_manager.pull().is_some() {}

        let ids = action_manager.clean_pending_entity_ids();
        assert_eq!(ids.len(), 0);

        action_manager.update(2500);
        while action_manager.pull().is_some() {}

        let ids = action_manager.clean_pending_entity_ids();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&1));
    }

    #[test]
    fn test_single_push_actions() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionClientManager<TestActionState> =
            ActionClientManager::new(data);

        action_manager.push_actions(vec![
            Arc::new(Action::new(1, 0, 1000, "Sleep")),
            Arc::new(Action::new(1, 1000, 3000, "Walk")),
        ]);
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep")),
            }),
        );
        asset_action_result(action_manager.pull(), None);

        action_manager.update(1500);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 1000, 3000, "Walk")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Update {
                action: Arc::new(Action::new(1, 1000, 3000, "Walk")),
                current_time: 1500,
            }),
        );
        asset_action_result(action_manager.pull(), None);
    }

    #[test]
    fn test_serialization_server() {
        let data = ActionManagerData::default();
        let mut action_manager1: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager1.attach(1);
        action_manager1.push_states(1, vec![("Sleep", 1000), ("Walk", 3000)]);
        let data = action_manager1.create_data();

        let mut action_manager2: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        while let Some(command) = action_manager1.pull() {
            if let ActionResult::Enqueue { .. } = command {
                continue;
            }
            asset_action_result(Some(command), action_manager2.pull());
        }
        asset_action_result(action_manager2.pull(), None);
    }

    #[test]
    fn test_serialization_client() {
        let data = ActionManagerData::default();
        let mut action_manager1: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager1.attach(1);
        action_manager1.push_states(1, vec![("Sleep", 1000), ("Walk", 3000)]);
        let data = action_manager1.create_data();

        let mut action_manager2: ActionClientManager<TestActionState> =
            ActionClientManager::new(data);

        while let Some(command) = action_manager1.pull() {
            if let ActionResult::Enqueue { .. } = command {
                continue;
            }
            asset_action_result(Some(command), action_manager2.pull());
        }
        asset_action_result(action_manager2.pull(), None);
    }

    #[test]
    fn test_client_lazy_attach_and_detach() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionClientManager<TestActionState> =
            ActionClientManager::new(data);

        action_manager.push_actions(vec![
            Arc::new(Action::new(1, 0, 1000, "Sleep")),
            Arc::new(Action::new(1, 1000, 1000, "Spawn")),
            Arc::new(Action::new(1, 1000, 2000, "Eat")),
            Arc::new(Action::new(2, 1000, 2000, "Sleep")),
        ]);

        action_manager.update(500);
        while action_manager.pull().is_some() {}

        action_manager.update(1000);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 1000, 1000, "Spawn")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 1000, 1000, "Spawn")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 1000, 2000, "Eat")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(2, 1000, 2000, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Update {
                action: Arc::new(Action::new(1, 1000, 2000, "Eat")),
                current_time: 1500,
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Update {
                action: Arc::new(Action::new(2, 1000, 2000, "Sleep")),
                current_time: 1500,
            }),
        );
        asset_action_result(action_manager.pull(), None);
    }

    #[test]
    fn test_client_cancel_immediate() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionClientManager<TestActionState> =
            ActionClientManager::new(data);

        action_manager.push_actions(vec![
            Arc::new(Action::new(1, 0, 1000, "Sleep")),
            Arc::new(Action::new(1, 500, 2000, "Spawn")),
        ]);

        while action_manager.pull().is_some() {}

        action_manager.update(1000);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Cancel {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep")),
            }),
        );
        assert_eq!(action_manager.current_time, 500);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 500, 2000, "Spawn")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Update {
                action: Arc::new(Action::new(1, 500, 2000, "Spawn")),
                current_time: 1000,
            }),
        );
        asset_action_result(action_manager.pull(), None);

        action_manager.update(1000);
        while action_manager.pull().is_some() {}

        assert_eq!(action_manager.last_actions.len(), 0);
    }

    #[test]
    fn test_server_cancel_zero_time() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);
        action_manager.push_states(1, vec![("Sleep", 0), ("Walk", 3000)]);

        action_manager.cancel(1, false);
        action_manager.push_states(1, vec![("Interrupt", 0)]);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 0, 0, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 0, 3000, "Walk")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 0, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 0, 0, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 3000, "Walk")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Cancel {
                action: Arc::new(Action::new(1, 0, 3000, "Walk")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 0, 0, "Interrupt")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 0, "Interrupt")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 0, 0, "Interrupt")),
            }),
        );
        asset_action_result(action_manager.pull(), None);
    }

    #[test]
    fn test_server_cancel_immediate_false_true() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);
        action_manager.push_states(1, vec![("Sleep", 0), ("Walk", 3000)]);

        action_manager.cancel(1, false);
        action_manager.cancel(1, true);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 0, 0, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 0, 3000, "Walk")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 0, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 0, 0, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 3000, "Walk")),
            }),
        );
    }

    #[test]
    fn test_server_cancel_immediate_true_false() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);
        action_manager.push_states(1, vec![("Sleep", 0), ("Walk", 3000)]);

        action_manager.cancel(1, true);
        action_manager.cancel(1, false);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 0, 0, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Enqueue {
                action: Arc::new(Action::new(1, 0, 3000, "Walk")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 0, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 0, 0, "Sleep")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1, 0, 3000, "Walk")),
            }),
        );
    }

    #[test]
    fn test_server_cancel_immediate_multiple() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1040);
        action_manager.attach(1039);
        action_manager.push_states(
            1040,
            vec![("WalkTo1", 500), ("WalkTo2", 500), ("WalkTo3", 500)],
        );
        action_manager.push_states(1039, vec![("WalkTo1", 500), ("Wait", 3000)]);

        while action_manager.pull().is_some() {}
        action_manager.update(1000);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1040, 0, 500, "WalkTo1")),
            }),
        );
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Start {
                action: Arc::new(Action::new(1040, 500, 1000, "WalkTo2")),
            }),
        );
        action_manager.cancel(1039, true);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1039, 0, 500, "WalkTo1")),
            }),
        );
    }

    #[test]
    fn test_same_time_cancel() {
        let data = ActionManagerData::default();
        let mut action_manager: ActionServerManager<TestActionState> =
            ActionServerManager::new(data);

        action_manager.attach(1);
        action_manager.attach(2);

        action_manager.push_states(1, vec![("Sleep1", 1000)]);
        action_manager.push_states(2, vec![("Sleep2", 1000)]);

        action_manager.update(0);
        while action_manager.pull().is_some() {}

        action_manager.update(1000);
        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::End {
                action: Arc::new(Action::new(1, 0, 1000, "Sleep1")),
            }),
        );
        action_manager.cancel(2, true);

        asset_action_result(
            action_manager.pull(),
            Some(ActionResult::Cancel {
                action: Arc::new(Action::new(2, 0, 1000, "Sleep2")),
            }),
        );

        asset_action_result(action_manager.pull(), None);
    }
}
