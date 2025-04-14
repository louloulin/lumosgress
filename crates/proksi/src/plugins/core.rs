use std::{collections::HashMap, sync::Arc};
use anyhow::Result;
use async_trait::async_trait;
use pingora::{http::{RequestHeader, ResponseHeader}, proxy::Session};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use thiserror::Error;
use strum_macros::{Display, EnumString};

use crate::proxy_server::https_proxy::RouterContext;
use crate::proxy_server::HttpResponse;

/// Defines the different processing stages for a plugin
#[derive(PartialEq, Debug, Default, Clone, Copy, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum PluginStep {
    /// Early phase before routing (first processing stage)
    EarlyRequest,
    
    /// Standard request processing (after routing)
    #[default]
    Request,
    
    /// Before proxying to upstream
    ProxyUpstream,
    
    /// Response headers processing
    Response,
    
    /// Response body processing
    ResponseBody,
    
    /// Logging phase (last processing stage)
    Log,
}

/// Plugin type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// Native Rust plugin compiled with proksi
    Native,
    
    /// Dynamic loaded Rust plugin
    Dynamic,
    
    /// WebAssembly plugin
    WebAssembly,
}

impl Default for PluginType {
    fn default() -> Self {
        Self::Native
    }
}

/// Plugin error types
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("{0} already exists")]
    AlreadyExists(&'static str),
    
    #[error("Plugin {0} not found")]
    NotFound(String),
    
    #[error("Failed to start plugin: {0}")]
    StartFailed(String),
    
    #[error("Failed to stop plugin: {0}")]
    StopFailed(String),
    
    #[error("WebAssembly execution error: {0}")]
    WasmError(String),
    
    #[error("Plugin configuration error: {0}")]
    ConfigError(String),
}

/// Plugin metadata containing information about the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    
    /// Plugin version
    pub version: String,
    
    /// Priority (lower values run first)
    pub priority: i32,
    
    /// Plugin type
    pub plugin_type: PluginType,
    
    /// Description
    #[serde(default)]
    pub description: String,
    
    /// Author information
    #[serde(default)]
    pub author: String,
    
    /// Plugin homepage URL
    #[serde(default)]
    pub homepage: Option<String>,
}

/// Unified Plugin interface
#[async_trait]
pub trait Plugin: Send + Sync + Debug {
    /// Returns plugin type (Native, Dynamic or WebAssembly)
    fn plugin_type(&self) -> PluginType {
        PluginType::Native
    }
    
    /// Returns a unique identifier for this plugin instance
    fn hash_key(&self) -> String {
        // Default implementation returns the plugin name
        self.name().to_string()
    }
    
    /// Returns the name of this plugin
    fn name(&self) -> &'static str;
    
    /// Returns metadata about this plugin
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 0,
            plugin_type: self.plugin_type(),
            description: String::new(),
            author: String::new(),
            homepage: None,
        }
    }
    
    /// Process HTTP requests at a specified step 
    /// 
    /// Returns:
    /// - `(handled, response)` tuple
    /// - `handled`: true if the plugin handled the request and prevented further processing
    /// - `response`: optional HTTP response to return to the client
    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // Default implementation: pass through
        Ok((false, None))
    }
    
    /// Process HTTP responses
    ///
    /// Returns:
    /// - `modified`: true if the response was modified
    async fn handle_response(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
        upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        // Default implementation: pass through
        Ok(false)
    }
    
    /// Start the plugin (initialization)
    async fn start(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// Stop the plugin (cleanup)
    async fn stop(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Plugin factory interface for creating plugin instances
pub trait PluginFactory: Send + Sync {
    /// Create a new plugin instance
    fn create(&self, config: &Value) -> Result<Box<dyn Plugin>, PluginError>;
    
    /// Get plugin name
    fn name(&self) -> &'static str;
    
    /// Get plugin type
    fn plugin_type(&self) -> PluginType {
        PluginType::Native
    }
    
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 0,
            plugin_type: self.plugin_type(),
            description: String::new(),
            author: String::new(),
            homepage: None,
        }
    }
}

/// Plugin registry for managing plugin factories and instances
pub struct PluginRegistry {
    /// Plugin factories
    factories: HashMap<String, Box<dyn PluginFactory>>,
    /// Plugin instances (cached)
    instances: HashMap<String, Arc<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            instances: HashMap::new(),
        }
    }
    
    /// Register a plugin factory
    pub fn register_factory(&mut self, factory: Box<dyn PluginFactory>) -> Result<(), PluginError> {
        let name = factory.name().to_string();
        if self.factories.contains_key(&name) {
            return Err(PluginError::AlreadyExists(
                Box::leak(name.into_boxed_str())
            ));
        }
        self.factories.insert(name, factory);
        Ok(())
    }
    
    /// Get a list of all registered plugin factories
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.factories.values()
            .map(|factory| factory.metadata())
            .collect()
    }
    
    /// Create or retrieve a plugin instance
    pub async fn get_or_create_plugin(
        &mut self, 
        name: &str, 
        config: &Value
    ) -> Result<Arc<dyn Plugin>, PluginError> {
        // Check if we already have an instance
        if let Some(instance) = self.instances.get(name) {
            return Ok(instance.clone());
        }
        
        // Try to create a new instance
        if let Some(factory) = self.factories.get(name) {
            let plugin = factory.create(config)?;
            let plugin_arc: Arc<dyn Plugin> = Arc::from(plugin);
            self.instances.insert(name.to_string(), plugin_arc.clone());
            Ok(plugin_arc)
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }
    
    /// Remove a plugin instance from the registry
    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        if self.instances.remove(name).is_some() {
            Ok(())
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }
} 