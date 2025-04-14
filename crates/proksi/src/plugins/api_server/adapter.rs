use std::sync::Arc;
use anyhow::Result;
use serde_json::Value;
use async_trait::async_trait;
use tracing::info;

use crate::plugins::core::{Plugin, PluginType, PluginError, PluginFactory, PluginMetadata, PluginStep};
use crate::plugins::api_server::{ApiServerPlugin, ApiServerConfig};
use crate::proxy_server::https_proxy::{RouterContext, HttpResponse};
use crate::config::Config;

/// Adapter for ApiServerPlugin to the new Plugin interface
#[derive(Debug)]
pub struct ApiServerPluginAdapter {
    inner: ApiServerPlugin,
}

#[async_trait]
impl Plugin for ApiServerPluginAdapter {
    fn plugin_type(&self) -> PluginType {
        PluginType::Native
    }
    
    fn name(&self) -> &'static str {
        "api_server"
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 0, // System plugin, should run early
            plugin_type: self.plugin_type(),
            description: "API Server plugin for Proksi".to_string(),
            author: "Proksi Team".to_string(),
            homepage: Some("https://github.com/proksi/proksi".to_string()),
        }
    }
    
    async fn handle_request(
        &self,
        _step: PluginStep,
        _session: &mut pingora::proxy::Session,
        _ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // API server doesn't handle requests directly
        Ok((false, None))
    }
    
    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting API server plugin");
        self.inner.start().await
    }
    
    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping API server plugin");
        self.inner.stop().await
    }
}

/// Factory for creating ApiServerPlugin instances
pub struct ApiServerPluginFactory {
    system_config: Arc<Config>,
}

impl ApiServerPluginFactory {
    pub fn new(system_config: Arc<Config>) -> Self {
        Self { system_config }
    }
}

impl PluginFactory for ApiServerPluginFactory {
    fn create(&self, config: &Value) -> Result<Box<dyn Plugin>, PluginError> {
        // Deserialize configuration
        let api_config: ApiServerConfig = serde_json::from_value(config.clone())
            .map_err(|e| PluginError::ConfigError(format!("Failed to parse API server config: {}", e)))?;
        
        // Create inner plugin
        let inner_plugin = ApiServerPlugin::new(api_config)
            .await
            .map_err(|e| PluginError::InitializationFailed(format!("Failed to create API server plugin: {:?}", e)))?
            .with_system_config(self.system_config.clone());
        
        // Wrap with adapter
        Ok(Box::new(ApiServerPluginAdapter { inner: inner_plugin }))
    }
    
    fn name(&self) -> &'static str {
        "api_server"
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 0,
            plugin_type: PluginType::Native,
            description: "API Server plugin for Proksi".to_string(),
            author: "Proksi Team".to_string(),
            homepage: Some("https://github.com/proksi/proksi".to_string()),
        }
    }
} 