use std::sync::Arc;
use dashmap::DashMap;
use async_trait::async_trait;

pub struct PluginManager {
    plugins: DashMap<&'static str, Arc<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: DashMap::new(),
        }
    }

    pub fn register<P: Plugin + 'static>(&self, plugin: P) {
        self.plugins.insert(plugin.name(), Arc::new(plugin));
    }

    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(name).map(|p| p.value().clone())
    }
} 