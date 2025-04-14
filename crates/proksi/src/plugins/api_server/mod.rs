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

use crate::plugins::core::{Plugin, PluginError};
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
    #[serde(default)]
    pub access_log: bool,
    
    /// Whether to enable CORS, default is true
    #[serde(default)]
    pub cors: bool,
}

fn default_listen_addr() -> String {
    "127.0.0.1:8080".to_string()
}

/// API Server Plugin for providing HTTP API endpoints
#[derive(Debug)]
pub struct ApiServerPlugin {
    config: ApiServerConfig,
    handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl ApiServerPlugin {
    /// Create a new API server plugin
    pub fn new(config: ApiServerConfig) -> Self {
        ApiServerPlugin {
            config,
            handle: None,
            shutdown_tx: None,
        }
    }

    /// Set the system configuration
    pub fn with_system_config(config: &Config) -> Self {
        let api_server_config = config.api_server.clone().unwrap_or_default();
        Self::new(api_server_config)
    }
}

#[async_trait]
impl Plugin for ApiServerPlugin {
    fn name(&self) -> &'static str {
        "api_server"
    }
    
    /// Start the API server
    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting API server on {}", self.config.listen_addr);
        
        let listen_addr = self.config.listen_addr.clone();
        let access_log = self.config.access_log;
        let cors = self.config.cors;

        let app_state = Arc::new(AppState {
            data: Mutex::new(HashMap::new()),
        });

        let mut router = Router::new()
            .route("/health", get(health_handler))
            .route("/metrics", get(metrics_handler))
            .route("/config", get(config_handler))
            .with_state(app_state);

        if access_log {
            // Add access logging if enabled
            warn!("Access logging for API server is enabled but not implemented");
        }

        if cors {
            // Add CORS if enabled
            warn!("CORS for API server is enabled but not implemented");
        }

        let addr: SocketAddr = listen_addr.parse()
            .map_err(|e| {
                error!("Failed to parse listen address: {}", e);
                PluginError::InitializationFailed(format!("Failed to parse listen address: {}", e))
            })?;
        
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);
        
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| {
                error!("Failed to bind to address: {}", e);
                PluginError::InitializationFailed(format!("Failed to bind to address: {}", e))
            })?;
        
        let server = axum::serve(listener, router);
        let server_with_shutdown = server.with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });
        
        let handle = tokio::spawn(async move {
            if let Err(e) = server_with_shutdown.await {
                error!("API server error: {}", e);
            }
        });
        
        self.handle = Some(handle);
        info!("API server started on {}", addr);
        
        Ok(())
    }
    
    /// Stop the API server
    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping API server");
        
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        
        if let Some(handle) = self.handle.take() {
            let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
        }
        
        info!("API server stopped");
        Ok(())
    }
}

/// Application state containing shared data
#[derive(Clone)]
struct AppState {
    data: Mutex<HashMap<String, i64>>,
}

/// Health check endpoint
async fn health_handler() -> &'static str {
    "OK"
}

/// Get metrics endpoint
async fn metrics_handler(State(state): State<AppState>) -> Json<HashMap<String, i64>> {
    let metrics = state.data.lock().await;
    Json(metrics.clone())
}

/// Get configuration endpoint
async fn config_handler(State(state): State<AppState>) -> impl IntoResponse {
    // In a real project, you might want to filter sensitive information
    let data = state.data.lock().await.clone();
    match serde_json::to_string(&data) {
        Ok(config_json) => (StatusCode::OK, config_json).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize config").into_response(),
    }
}

/// Compatibility function to start API server using existing code
pub async fn start_api_server(config: Arc<Config>) -> Result<()> {
    let api_config = ApiServerConfig {
        listen_addr: "127.0.0.1:8080".to_string(),
        access_log: true,
        cors: true,
    };
    
    let mut plugin = ApiServerPlugin::new(api_config);
    
    plugin.start().await
        .map_err(|e| anyhow::anyhow!("Failed to start API server: {}", e))?;
    
    // Server runs in the background
    Ok(())
} 