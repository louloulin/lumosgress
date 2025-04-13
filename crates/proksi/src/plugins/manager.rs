use std::sync::Arc;
use dashmap::DashMap;
use async_trait::async_trait;
use once_cell::sync::Lazy;

use crate::plugins::{Plugin, PluginConfig};

// Global plugin manager instance
static PLUGIN_MANAGER: Lazy<PluginManager> = Lazy::new(PluginManager::new);

pub struct PluginManager {
    // Use a type-erased trait object with the Config type erased
    plugins: DashMap<&'static str, Arc<dyn std::any::Any + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: DashMap::new(),
        }
    }

    pub fn register<P>(&self, plugin: P)
    where
        P: Plugin + 'static,
    {
        self.plugins.insert(plugin.name(), Arc::new(plugin));
    }

    pub fn get_plugin<P, C>(&self, name: &str) -> Option<Arc<P>>
    where
        P: Plugin<Config = C> + 'static,
        C: PluginConfig,
    {
        self.plugins
            .get(name)
            .and_then(|p| p.value().clone().downcast::<P>().ok())
    }
}

// Module-level function to register a plugin with the global manager
pub fn register<P>(plugin: P)
where
    P: Plugin + 'static,
{
    PLUGIN_MANAGER.register(plugin);
}

// Module-level function to get a plugin from the global manager
pub fn get_plugin<P, C>(name: &str) -> Option<Arc<P>>
where
    P: Plugin<Config = C> + 'static,
    C: PluginConfig,
{
    PLUGIN_MANAGER.get_plugin(name)
} 