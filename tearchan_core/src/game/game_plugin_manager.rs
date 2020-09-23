use crate::game::game_plugin::GamePlugin;
use std::collections::HashMap;
use std::ops::Deref;

pub type GamePluginId = *const dyn GamePlugin;

pub struct GamePluginManager {
    plugins: HashMap<GamePluginId, (Box<dyn GamePlugin>, i32)>,
    plugin_keys: HashMap<String, GamePluginId>,
    plugin_orders: Vec<GamePluginId>,
}

impl GamePluginManager {
    pub fn new() -> Self {
        GamePluginManager {
            plugins: HashMap::new(),
            plugin_orders: vec![],
            plugin_keys: HashMap::new(),
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

    pub fn get(&self, key: &str) -> Option<&Box<dyn GamePlugin>> {
        let id = self.plugin_keys.get(key)?;
        self.plugins.get(&id).map(|(plugin, _)| plugin)
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
}
