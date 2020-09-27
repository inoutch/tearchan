use crate::game::game_plugin::GamePlugin;
use crate::game::game_plugin_command::GamePluginCommand;
use crate::game::game_plugin_operator::GamePluginOperator;
use std::collections::{HashMap, VecDeque};
use std::ops::Deref;
use tearchan_utility::shared::Shared;

pub type GamePluginId = *const dyn GamePlugin;

pub struct GamePluginManager {
    plugins: HashMap<GamePluginId, (Box<dyn GamePlugin>, i32)>,
    plugin_keys: HashMap<String, GamePluginId>,
    plugin_orders: Vec<GamePluginId>,
    commands: Shared<VecDeque<GamePluginCommand>>,
}

impl GamePluginManager {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        GamePluginManager {
            plugins: HashMap::new(),
            plugin_orders: vec![],
            plugin_keys: HashMap::new(),
            commands: Shared::new(VecDeque::new()),
        }
    }

    pub fn add(&mut self, plugin: Box<dyn GamePlugin>, key: String, order: i32) {
        let id: GamePluginId = plugin.deref();
        self.plugins.insert(id, (plugin, order));
        self.plugin_keys.insert(key, id);

        let plugins = &self.plugins;
        self.plugin_orders.push(id);
        self.plugin_orders
            .sort_by(|a, b| plugins[a].1.cmp(&plugins[b].1))
    }

    pub fn remove(&mut self, key: String) -> Option<Box<dyn GamePlugin>> {
        let id = self.plugin_keys.remove(&key)?;
        self.plugin_orders
            .remove(self.plugin_orders.iter().position(|x| x == &id).unwrap());
        self.plugins.remove(&id).map(|(plugin, _)| plugin)
    }

    pub fn get(&self, key: &str) -> Option<&dyn GamePlugin> {
        let id = self.plugin_keys.get(key)?;
        self.plugins.get(&id).map(|(plugin, _)| plugin.deref())
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Box<dyn GamePlugin>> {
        let id = self.plugin_keys.get(key)?;
        self.plugins.get_mut(&id).map(|(plugin, _)| plugin)
    }

    pub fn for_each_mut<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Box<dyn GamePlugin>),
    {
        for plugin_order in &self.plugin_orders {
            let (plugin, _) = self.plugins.get_mut(plugin_order).unwrap();
            callback(plugin);
        }
    }

    pub fn create_operator(&self) -> GamePluginOperator {
        GamePluginOperator::new(Shared::clone(&self.commands))
    }

    pub fn update(&mut self) {
        let mut commands = self.commands.borrow_mut();
        while let Some(command) = commands.pop_back() {
            match command {
                GamePluginCommand::CreateGameObject { object } => {
                    for plugin_id in &self.plugin_orders {
                        let (plugin, _) = self.plugins.get_mut(plugin_id).unwrap();
                        plugin.on_add(&object);
                    }
                }
            }
        }
    }
}
