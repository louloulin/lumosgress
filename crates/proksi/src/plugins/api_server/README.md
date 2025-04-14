# API Server Plugin

The API Server Plugin provides HTTP API endpoints for monitoring, configuration retrieval, and health checking of the Proksi service.

## Features

- Health check endpoint (`/health`)
- Metrics collection endpoint (`/metrics`)
- Configuration retrieval endpoint (`/api/v1/config`)
- Configurable listen address
- Access logging support
- CORS support

## Configuration

The API Server Plugin can be configured using the following options:

```hcl
plugins {
  api_server {
    # Address to listen on, default is 127.0.0.1:8080
    listen_address = "127.0.0.1:8080"
    
    # Whether to enable access logging, default is true
    enable_access_log = true
    
    # Whether to enable CORS, default is true
    enable_cors = true
  }
}
```

## Endpoints

### Health Check

```
GET /health
```

Returns a simple "OK" response to confirm the server is running.

### Metrics

```
GET /metrics
```

Returns a JSON object containing metrics about the server, such as request counts and error counts.

### Configuration

```
GET /api/v1/config
```

Returns the current configuration of the server in JSON format.

## Usage in Code

```rust
use std::sync::Arc;
use crate::config::Config;
use crate::plugins::api_server::{ApiServerPlugin, ApiServerConfig};

async fn start_api_server(config: Arc<Config>) -> Result<()> {
    // Create configuration
    let api_config = ApiServerConfig {
        listen_address: "127.0.0.1:8080".to_string(),
        enable_access_log: true,
        enable_cors: true,
    };
    
    // Create and initialize plugin
    let mut plugin = ApiServerPlugin::new(api_config).await?;
    
    // Set system configuration
    plugin = plugin.with_system_config(config);
    
    // Start the API server
    plugin.start().await?;
    
    // Server runs in the background
    Ok(())
}
```

## Implementation Details

The plugin uses the [axum](https://github.com/tokio-rs/axum) web framework to provide HTTP endpoints. It runs in a separate Tokio task and can be gracefully shut down via the `stop()` method. 