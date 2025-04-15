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
    fn name(&self) -> &'static str {
        self.inner.name()
    }
    
    fn plugin_type(&self) -> PluginType {
        self.inner.plugin_type()
    }
    
    fn metadata(&self) -> PluginMetadata {
        self.inner.metadata()
    }
    
    async fn handle_request(
        &self,
        _step: PluginStep,
        _session: &mut pingora::proxy::Session,
        _ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // API server doesn't handle proxy requests
        Ok((false, None))
    }
    
    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting API server plugin adapter");
        self.inner.start().await
    }
    
    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping API server plugin adapter");
        self.inner.stop().await
    }
}

/// Factory for creating ApiServerPlugin instances
pub struct ApiServerPluginFactory;

impl ApiServerPluginFactory {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PluginFactory for ApiServerPluginFactory {
    async fn create_plugin(&self, config: Arc<Config>) -> Result<Arc<dyn Plugin>> {
        let api_config = config.plugins.as_ref()
            .and_then(|p| p.api_server.as_ref())
            .ok_or_else(|| PluginError::ConfigurationMissing("API server configuration not found".to_string()))?;

        if !api_config.enabled {
            return Err(PluginError::Disabled("API server plugin is disabled".to_string()));
        }

        let plugin_config = ApiServerConfig {
            listen_addr: api_config.listen_address.clone()
                .unwrap_or_else(|| "127.0.0.1:8080".to_string()),
            access_log: api_config.enable_access_log.unwrap_or(true),
            cors: api_config.enable_cors.unwrap_or(true),
        };

        let plugin = ApiServerPlugin::new(plugin_config).await?;
        let plugin = plugin.with_system_config(config);
        let adapter = ApiServerPluginAdapter { inner: plugin };

        Ok(Arc::new(adapter))
    }
}