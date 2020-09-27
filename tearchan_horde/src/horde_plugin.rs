use crate::action::action_manager::{ActionManager, ActionResult};
use crate::action_creator::action_creator_manager::{ActionCreatorCommand, ActionCreatorManager};
use crate::action_creator::action_creator_result::ActionCreatorResult;
use crate::object::object_error::ObjectError;
use crate::object::object_factory::ObjectFactory;
use crate::object::object_store::{ObjectStore, ObjectStoreBase};
use crate::object::Object;
use serde::export::fmt::Debug;
use std::collections::{HashMap, HashSet};
use tearchan_core::game::game_context::GameContext;
use tearchan_core::game::game_plugin::GamePlugin;
use tearchan_core::game::game_plugin_command::GamePluginCommand;
use tearchan_core::game::game_plugin_operator::GamePluginOperator;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::game_object_manager::GameObjectManager;
use tearchan_core::game::object::{GameObject, GameObjectId};

pub trait HordePluginProvider {
    type ActionCommonStore: Debug;
    type ActionCreatorCommonStore: Debug;

    fn on_start_action(&mut self, store: &Self::ActionCommonStore, object: GameObject<dyn Object>);

    fn on_update_action(
        &mut self,
        store: &Self::ActionCommonStore,
        current_time: u64,
        ratio: f32,
        object: GameObject<dyn Object>,
    );

    fn on_end_action(&mut self, store: &Self::ActionCommonStore, object: GameObject<dyn Object>);

    fn run_action_creator(
        &mut self,
        action_manager: &mut ActionManager<Self::ActionCommonStore>,
        store: &mut Self::ActionCreatorCommonStore,
        object: &mut GameObject<dyn Object>,
    ) -> ActionCreatorResult<Self::ActionCreatorCommonStore>;

    fn create_action_command(
        &mut self,
        priority: u32,
    ) -> ActionCreatorCommand<Self::ActionCreatorCommonStore>;
}

pub struct HordePlugin<T: HordePluginProvider> {
    action_manager: ActionManager<T::ActionCommonStore>,
    action_creator_manager: ActionCreatorManager<T::ActionCreatorCommonStore>,
    action_objects: HashSet<GameObjectId>,
    object_manager: GameObjectManager<dyn Object>,
    object_factories: HashMap<String, ObjectFactory>,
    object_stores: Vec<ObjectStore<dyn ObjectStoreBase>>,
    operator: GamePluginOperator,
    provider: T,
}

impl<T: HordePluginProvider> GamePlugin for HordePlugin<T> {
    fn on_add(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(horde_object) = game_object.cast::<dyn Object>() {
            self.action_objects.insert(horde_object.id());
            self.object_manager.add(horde_object);
        }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        if let Some(horde_object) = game_object.cast::<dyn Object>() {
            self.action_objects.remove(&horde_object.id());
            self.object_manager.remove(&horde_object.id());
        }
    }

    fn on_update(&mut self, context: &mut GameContext) {
        self.action_manager
            .update((context.delta * 1000.0f32) as u64);

        while {
            self.update_action();
            // self.object_manager.flush(object_context);

            let mut is_changed = false;
            // update all objects
            for object_id in &self.action_objects {
                is_changed = is_changed
                    || update_object(
                        *object_id,
                        &mut self.provider,
                        &mut self.action_manager,
                        &mut self.action_creator_manager,
                        &mut self.object_manager,
                    );
            }
            is_changed
        } {}

        // self.object_manager.update(object_context);
    }
}

impl<T: HordePluginProvider> HordePlugin<T> {
    pub fn new(provider: T, operator: GamePluginOperator) -> HordePlugin<T> {
        HordePlugin {
            action_manager: ActionManager::new(),
            action_creator_manager: ActionCreatorManager::new(),
            action_objects: HashSet::new(),
            object_manager: GameObjectManager::new(),
            object_factories: HashMap::new(),
            object_stores: vec![],
            operator,
            provider,
        }
    }

    pub fn register_factory(&mut self, kind: &str, factory: ObjectFactory) {
        debug_assert!(
            !self.object_factories.contains_key(kind),
            format!(
                "The kind of factory already has been registered [kind = {}]",
                kind
            )
        );
        self.object_factories.insert(kind.to_string(), factory);
    }

    pub fn create_object(
        &mut self,
        store: ObjectStore<dyn ObjectStoreBase>,
    ) -> Result<GameObjectId, ObjectError> {
        self.create_object_will_full_args(store, None)
    }

    fn create_object_will_full_args(
        &mut self,
        mut store: ObjectStore<dyn ObjectStoreBase>,
        parent_id: Option<GameObjectId>,
    ) -> Result<GameObjectId, ObjectError> {
        // Select factory phase
        let factory = self
            .object_factories
            .get(store.kind())
            .ok_or(ObjectError::FactoryNotRegistered)?;

        // Select parent from args or store
        let merged_parent_id = parent_id.or_else(|| store.parent_id());
        let parent = merged_parent_id
            .and_then(|parent_id| {
                self.object_manager
                    .find_by_id(parent_id)
                    .map(|object| (object, parent_id))
            })
            .map(|(object, parent_id)| {
                store.set_parent_id(parent_id);
                object
            });

        let object = factory((store.clone(), parent, self.object_manager.create_operator()))
            .ok_or(ObjectError::CreationFailed)?;
        let object_id = object.id();
        store.set_id(object_id);

        let game_object = object.cast().ok_or(ObjectError::InvalidType)?;
        self.operator.queue(GamePluginCommand::CreateGameObject {
            object: game_object,
        });
        self.object_stores.push(store);

        Ok(object_id)
    }

    fn update_action(&mut self) {
        let mut results = self.action_manager.pull();
        while let Some(result) = results.pop_first_back() {
            let object = self.object_manager.find_by_id(result.object_id()).unwrap();

            match result {
                ActionResult::Start { store } => {
                    self.provider.on_start_action(store.common(), object);
                }
                ActionResult::Update {
                    store,
                    current_time,
                } => {
                    self.provider.on_update_action(
                        store.common(),
                        current_time,
                        store.ratio(current_time),
                        object,
                    );
                }
                ActionResult::End { store } => {
                    self.provider.on_end_action(store.common(), object);
                }
            }
        }
    }
}

fn update_object<T: HordePluginProvider>(
    object_id: GameObjectId,
    provider: &mut T,
    action_manager: &mut ActionManager<T::ActionCommonStore>,
    action_creator_manager: &mut ActionCreatorManager<T::ActionCreatorCommonStore>,
    object_manager: &mut GameObjectManager<dyn Object>,
) -> bool {
    let mut object = object_manager.find_by_id(object_id).unwrap();
    let object_id = object.id();

    if action_manager.is_running(&object_id) {
        return false;
    }

    let mut command_priority = 0u32;
    let mut next_cmd: Option<ActionCreatorCommand<T::ActionCreatorCommonStore>> =
        if action_creator_manager.is_running(&object_id) {
            Some(ActionCreatorCommand::Rerun)
        } else {
            None
        };
    loop {
        let cmd = match next_cmd.take() {
            Some(x) => x,
            None => {
                let command = provider.create_action_command(command_priority);
                command_priority += 1;
                command
            }
        };

        let result = match action_creator_manager.run(object_id, cmd) {
            Some(x) => x,
            None => {
                continue;
            }
        };
        let next = provider.run_action_creator(action_manager, result.store_mut(), &mut object);
        match next {
            ActionCreatorResult::Continue { command } => {
                next_cmd = Some(command);
                continue;
            }
            ActionCreatorResult::Break => {
                if action_manager.is_running(&object_id) {
                    return true;
                }
                next_cmd = None;
            }
        }
    }
}
