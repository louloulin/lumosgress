use std::sync::Arc;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{info, error};
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

use crate::plugins::{Plugin, PluginConfig, PluginError};
use crate::config::Config;

#[cfg(test)]
mod tests;

/// Configuration for the API server plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServerConfig {
    /// Address to listen on, default is 127.0.0.1:8080
    #[serde(default = "default_listen_address")]
    pub listen_address: String,
    
    /// Whether to enable access logging, default is true
    #[serde(default = "default_enable_access_log")]
    pub enable_access_log: bool,
    
    /// Whether to enable CORS, default is true
    #[serde(default = "default_enable_cors")]
    pub enable_cors: bool,
}

fn default_listen_address() -> String {
    "127.0.0.1:8080".to_string()
}

fn default_enable_access_log() -> bool {
    true
}

fn default_enable_cors() -> bool {
    true
}

impl PluginConfig for ApiServerConfig {}

/// API Server Plugin for providing HTTP API endpoints
pub struct ApiServerPlugin {
    config: Arc<ApiServerConfig>,
    system_config: Arc<Config>,
    server_handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    metrics: Arc<Mutex<HashMap<String, i64>>>,
}

#[async_trait]
impl Plugin for ApiServerPlugin {
    type Config = ApiServerConfig;
    
    async fn new(config: Self::Config) -> Result<Self, PluginError> {
        // Create initial metrics
        let mut metrics = HashMap::new();
        metrics.insert("requests".to_string(), 0);
        metrics.insert("errors".to_string(), 0);
        
        Ok(Self {
            config: Arc::new(config),
            system_config: Arc::new(Config::default()),
            server_handle: None,
            shutdown_tx: None,
            metrics: Arc::new(Mutex::new(metrics)),
        })
    }
    
    fn name(&self) -> &'static str {
        "api_server"
    }
}

impl ApiServerPlugin {
    /// Set the system configuration
    pub fn with_system_config(mut self, config: Arc<Config>) -> Self {
        self.system_config = config;
        self
    }
    
    /// Start the API server
    pub async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting API server on {}", self.config.listen_address);
        
        let metrics = self.metrics.clone();
        let system_config = self.system_config.clone();
        
        // Create routes
        let app = Router::new()
            .route("/", get(health_check))
            .route("/health", get(health_check))
            .route("/metrics", get(get_metrics))
            .route("/api/v1/config", get(get_config))
            .with_state(AppState { 
                metrics, 
                system_config 
            });
        
        // Parse listen address
        let addr: SocketAddr = self.config.listen_address.parse()
            .map_err(|e| PluginError::InitializationFailed(format!("Failed to parse listen address: {}", e)))?;
        
        // Create shutdown channel
        let (tx, rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(tx);
        
        // Start server
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| PluginError::InitializationFailed(format!("Failed to bind to address: {}", e)))?;
        
        let server = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
                info!("API server shutdown signal received");
            });
        
        let handle = tokio::spawn(async move {
            if let Err(e) = server.await {
                error!("API server error: {}", e);
            }
            info!("API server stopped");
        });
        
        self.server_handle = Some(handle);
        info!("API server plugin started");
        
        Ok(())
    }
    
    /// Stop the API server
    pub async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping API server");
        
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        
        if let Some(handle) = self.server_handle.take() {
            match tokio::time::timeout(Duration::from_secs(5), handle).await {
                Ok(result) => {
                    if let Err(e) = result {
                        error!("Error joining API server handle: {}", e);
                    }
                },
                Err(_) => {
                    error!("Timeout waiting for API server to stop");
                }
            }
        }
        
        Ok(())
    }
}

/// Application state containing shared data
#[derive(Clone)]
struct AppState {
    metrics: Arc<Mutex<HashMap<String, i64>>>,
    system_config: Arc<Config>,
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Get metrics endpoint
async fn get_metrics(State(state): State<AppState>) -> Json<HashMap<String, i64>> {
    let metrics = state.metrics.lock().await;
    Json(metrics.clone())
}

/// Get configuration endpoint
async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    // In a real project, you might want to filter sensitive information
    match serde_json::to_string(&*state.system_config) {
        Ok(config_json) => (StatusCode::OK, config_json).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize config").into_response(),
    }
}

/// Compatibility function to start API server using existing code
pub async fn start_api_server(config: Arc<Config>) -> Result<()> {
    let api_config = ApiServerConfig {
        listen_address: "127.0.0.1:8080".to_string(),
        enable_access_log: true,
        enable_cors: true,
    };
    
    let mut plugin = ApiServerPlugin::new(api_config).await
        .map_err(|e| anyhow::anyhow!("Failed to create API server plugin: {}", e))?;
    
    plugin = plugin.with_system_config(config);
    plugin.start().await
        .map_err(|e| anyhow::anyhow!("Failed to start API server: {}", e))?;
    
    // Server runs in the background
    Ok(())
} 