use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use pingora::proxy::Session;
use pingora::http::ResponseHeader;
use http::HeaderValue;
use std::borrow::Cow;
use tracing::{debug, info, warn};

use crate::proxy_server::https_proxy::RouterContext;
use crate::proxy_server::HttpResponse;
use crate::plugins::core::{Plugin, PluginError, PluginStep, PluginMetadata, PluginType};
use uuid::Uuid;

#[cfg(test)]
mod tests;

/// Configuration for the RequestId plugin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestIdConfig {
    /// Whether to enable the request ID generation
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// The header name to use for the request ID in responses
    #[serde(default = "default_header_name")]
    pub header_name: String,

    /// Whether to preserve existing request IDs from incoming requests
    #[serde(default = "default_true")]
    pub preserve_existing: bool,

    /// Whether to add request ID to request headers for upstream
    #[serde(default = "default_false")]
    pub add_to_upstream: bool,
}

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}

fn default_header_name() -> String {
    "x-request-id".to_string()
}

/// A plugin that adds a request ID to the request headers
/// and response headers
#[derive(Debug)]
pub struct RequestId {
    config: RequestIdConfig,
}

impl Default for RequestIdConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            header_name: default_header_name(),
            preserve_existing: true,
            add_to_upstream: false,
        }
    }
}

impl RequestId {
    /// Create a new RequestId plugin with default configuration
    pub fn new() -> Self {
        info!("Initializing RequestId plugin with default configuration");
        Self {
            config: RequestIdConfig::default(),
        }
    }

    /// Create a new RequestId plugin with custom configuration
    pub fn with_config(config: RequestIdConfig) -> Self {
        info!("Initializing RequestId plugin with custom configuration");
        Self { config }
    }
}

#[async_trait]
impl Plugin for RequestId {
    fn name(&self) -> &'static str {
        "request_id"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 10, // Request ID should run very early
            plugin_type: PluginType::Native,
            description: "Generates unique request IDs for tracing and debugging".to_string(),
            author: "Proksi Team".to_string(),
            homepage: Some("https://github.com/luizfonseca/proksi".to_string()),
        }
    }
    
    /// Generate a unique request ID and add it to the context
    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // Only process during EarlyRequest step to ensure request ID is available early
        if step != PluginStep::EarlyRequest {
            return Ok((false, None));
        }

        // Only generate request ID if the plugin is enabled
        if !self.config.enabled {
            debug!("RequestId plugin is disabled, skipping");
            return Ok((false, None));
        }

        // Check if there's already a request ID in the incoming headers
        let existing_request_id = session.req_header().headers
            .get(&self.config.header_name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Determine the request ID to use
        let request_id = if let Some(existing_id) = existing_request_id {
            if self.config.preserve_existing && !existing_id.is_empty() {
                debug!("Using existing request ID: {}", existing_id);
                existing_id
            } else {
                let new_id = Uuid::new_v4().to_string();
                debug!("Replacing existing request ID with new one: {}", new_id);
                new_id
            }
        } else {
            let new_id = Uuid::new_v4().to_string();
            debug!("Generated new request ID: {}", new_id);
            new_id
        };

        // Set the request ID in the context
        ctx.request_id = request_id.clone();

        // Store in extensions for tests and other plugins
        ctx.extensions.insert(Cow::Borrowed("request_id_header"), request_id.clone());

        // Add to upstream request headers if configured
        if self.config.add_to_upstream {
            if let Ok(header_value) = HeaderValue::from_str(&request_id) {
                session.req_header_mut().insert_header(self.config.header_name.clone(), header_value)
                    .map_err(|e| {
                        warn!("Failed to add request ID to upstream headers: {}", e);
                        e
                    })?;
                debug!("Added request ID to upstream headers");
            }
        }

        // Continue processing
        Ok((false, None))
    }
    
    /// Add the request ID to the response headers
    async fn handle_response(
        &self,
        _step: PluginStep,
        _session: &mut Session,
        ctx: &mut RouterContext,
        upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        // Only add the header if the plugin is enabled
        if !self.config.enabled {
            return Ok(false);
        }

        // Get the request ID from the context
        if !ctx.request_id.is_empty() {
            if let Ok(header_value) = HeaderValue::from_str(&ctx.request_id) {
                // Create a local String variable to avoid lifetime issues
                let header_name = self.config.header_name.clone();
                
                // Add the request ID to the response headers using the header string 
                // directly, not as a reference
                upstream_response.insert_header(header_name, header_value)?;
                return Ok(true);
            }
        }

        Ok(false)
    }
    
    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting RequestId plugin");
        if !self.config.enabled {
            info!("RequestId plugin is disabled");
        } else {
            info!("RequestId plugin started with header: {}", self.config.header_name);
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping RequestId plugin");
        Ok(())
    }
}