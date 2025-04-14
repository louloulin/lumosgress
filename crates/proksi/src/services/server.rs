use std::sync::Arc;
use anyhow::Result;

use crate::config::Config;
use crate::plugins::api_server;

// The API Server implementation has been migrated to the plugin system.
// This file is kept for backward compatibility until all dependencies are updated.
// Please use crate::plugins::api_server module instead.

/// Start the API server using the plugin implementation
pub async fn start_api_server(config: Arc<Config>) -> Result<()> {
    api_server::start_api_server(config).await
} 