use std::sync::Arc;
use anyhow::Result;
use pingora::proxy::ProxyHttp;
use pingora_core::services::Service;

use crate::config::Config;
use crate::plugins::manager::PluginManager;

pub fn create_server(
    config: Arc<Config>,
    plugin_manager: Arc<PluginManager>,
) -> Result<Box<dyn Service>> {
    // Initialize proxy service with config and plugins
    Ok(Box::new(HttpServer {
        config,
        plugin_manager,
    }))
}

pub struct HttpServer {
    config: Arc<Config>,
    plugin_manager: Arc<PluginManager>,
}