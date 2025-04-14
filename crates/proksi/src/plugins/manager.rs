use std::sync::Arc;
use dashmap::DashMap;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use anyhow::Result;

use crate::plugins::{Plugin, PluginConfig, PluginError};

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
        P: Plugin + Send + Sync + 'static,
    {
        self.plugins.insert(plugin.name(), Arc::new(plugin));
    }

    pub fn get_plugin<P>(&self, name: &str) -> Option<Arc<P>>
    where
        P: Plugin + Send + Sync + 'static,
    {
        self.plugins
            .get(name)
            .and_then(|p| p.value().clone().downcast::<P>().ok())
    }
    
    // 根据名称判断插件是否已注册
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }
    
    // 获取所有已注册的插件名称
    pub fn list_plugins(&self) -> Vec<&'static str> {
        self.plugins.iter().map(|entry| *entry.key()).collect()
    }
    
    // 停止并卸载插件
    pub async fn unload_plugin(&self, name: &str) -> Result<(), PluginError> {
        // 首先检查插件是否存在
        if !self.has_plugin(name) {
            return Err(PluginError::NotFound(name.to_string()));
        }
        
        // 查找对应的静态字符串key
        let static_key = self.plugins.iter()
            .find(|entry| *entry.key() == name)
            .map(|entry| *entry.key())
            .expect("Plugin exists but couldn't find matching key");
            
        // 移除插件
        self.plugins.remove(&static_key);
        Ok(())
    }
}

// Module-level function to register a plugin with the global manager
pub fn register<P>(plugin: P)
where
    P: Plugin + Send + Sync + 'static,
{
    PLUGIN_MANAGER.register(plugin);
}

// Module-level function to get a plugin from the global manager
pub fn get_plugin<P>(name: &str) -> Option<Arc<P>>
where
    P: Plugin + Send + Sync + 'static,
{
    PLUGIN_MANAGER.get_plugin(name)
}

// 判断插件是否已注册
pub fn has_plugin(name: &str) -> bool {
    PLUGIN_MANAGER.has_plugin(name)
}

// 获取所有已注册插件的名称
pub fn list_plugins() -> Vec<&'static str> {
    PLUGIN_MANAGER.list_plugins()
}

// 停止并卸载插件
pub async fn unload_plugin(name: &str) -> Result<(), PluginError> {
    PLUGIN_MANAGER.unload_plugin(name).await
} 