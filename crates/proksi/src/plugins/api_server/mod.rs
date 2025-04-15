use std::sync::Arc;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{info, error, warn};
use axum::{
    routing::get,
    Router, 
    extract::State,
    http::StatusCode,
    response::{Json, IntoResponse},
};
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::net::TcpListener;

use crate::plugins::core::{Plugin, PluginError, PluginType, PluginMetadata};
use crate::config::Config;

#[cfg(test)]
mod tests;

/// Configuration for the API server plugin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiServerConfig {
    /// Address to listen on, default is 127.0.0.1:8080
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    
    /// Whether to enable access logging, default is true
    #[serde(default = "default_true")]
    pub access_log: bool,
    
    /// Whether to enable CORS, default is true
    #[serde(default = "default_true")]
    pub cors: bool,
}

fn default_true() -> bool {
    true
}

fn default_listen_addr() -> String {
    "127.0.0.1:8080".to_string()
}

/// API Server Plugin for providing HTTP API endpoints
#[derive(Debug)]
pub struct ApiServerPlugin {
    config: ApiServerConfig,
    system_config: Option<Arc<Config>>,
    server_handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

/// Application state containing shared data
#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    data: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl ApiServerPlugin {
    /// Create a new API server plugin
    pub async fn new(config: ApiServerConfig) -> Result<Self, PluginError> {
        Ok(Self {
            config,
            system_config: None,
            server_handle: None,
            shutdown_tx: None,
        })
    }

    /// Set the system configuration
    pub fn with_system_config(mut self, config: Arc<Config>) -> Self {
        self.system_config = Some(config);
        self
    }
}

#[async_trait]
impl Plugin for ApiServerPlugin {
    fn name(&self) -> &'static str {
        "api_server"
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Native
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 0,
            plugin_type: self.plugin_type(),
            description: "Provides HTTP API endpoints for monitoring and configuration".to_string(),
            author: "Proksi Team".to_string(),
            homepage: Some("https://github.com/proksi/proksi".to_string()),
        }
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting API server on {}", self.config.listen_addr);
        
        let system_config = self.system_config.clone().ok_or_else(|| {
            PluginError::InitializationFailed("System config not set".to_string())
        })?;

        let app_state = AppState {
            config: system_config,
            data: Arc::new(Mutex::new(HashMap::new())),
        };

        let mut router = Router::new()
            .route("/health", get(health_handler))
            .route("/metrics", get(metrics_handler))
            .route("/config", get(config_handler))
            .with_state(app_state);

        if self.config.access_log {
            // Add access logging middleware if enabled
            warn!("Access logging for API server is enabled but not implemented");
        }

        if self.config.cors {
            // Add CORS middleware if enabled
            warn!("CORS for API server is enabled but not implemented");
        }

        let addr: SocketAddr = self.config.listen_addr.parse()
            .map_err(|e| PluginError::InitializationFailed(format!("Invalid address: {}", e)))?;

        let listener = TcpListener::bind(&addr).await
            .map_err(|e| PluginError::InitializationFailed(format!("Failed to bind: {}", e)))?;

        let actual_addr = listener.local_addr()
            .map_err(|e| PluginError::InitializationFailed(format!("Failed to get local address: {}", e)))?;
        
        info!("API server listening on {}", actual_addr);

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let server = axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            });

        self.server_handle = Some(tokio::spawn(async move {
            if let Err(e) = server.await {
                error!("API server error: {}", e);
            }
        }));

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping API server");
        
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        Ok(())
    }
}

/// Health check endpoint handler
async fn health_handler() -> &'static str {
    "OK"
}

/// Metrics endpoint handler
async fn metrics_handler(State(state): State<AppState>) -> Json<HashMap<String, i64>> {
    // TODO: Implement real metrics collection
    let metrics = HashMap::from([
        ("requests_total".to_string(), 0),
        ("errors_total".to_string(), 0),
    ]);
    Json(metrics)
}

/// Config endpoint handler 
async fn config_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Return sanitized configuration
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}

/// Compatibility function to start API server using existing code
pub async fn start_api_server(config: Arc<Config>) -> Result<()> {
    if let Some(api_config) = &config.plugins.as_ref().and_then(|p| p.api_server.as_ref()) {
        if api_config.enabled {
            let plugin_config = ApiServerConfig {
                listen_addr: api_config.listen_address.clone()
                    .unwrap_or_else(default_listen_addr),
                access_log: api_config.enable_access_log.unwrap_or(true),
                cors: api_config.enable_cors.unwrap_or(true),
            };
            
            let mut plugin = ApiServerPlugin::new(plugin_config).await?;
            plugin = plugin.with_system_config(config.clone());
            
            plugin.start().await.map_err(|e| anyhow::anyhow!("Failed to start API server: {}", e))?;
        }
    }
    
    Ok(())
}