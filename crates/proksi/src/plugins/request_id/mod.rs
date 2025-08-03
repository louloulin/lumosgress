use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use pingora::proxy::Session;
use pingora::http::ResponseHeader;
use http::HeaderValue;
use std::borrow::Cow;

use crate::proxy_server::https_proxy::RouterContext;
use crate::proxy_server::HttpResponse;
use crate::plugins::core::{Plugin, PluginError, PluginStep};
use uuid::Uuid;

#[cfg(test)]
mod tests;

/// Configuration for the RequestId plugin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestIdConfig {
    /// Whether to enable the request ID
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// The header name to use for the request ID
    #[serde(default = "default_header_name")]
    pub header_name: String,
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

impl RequestId {
    pub fn new() -> Self {
        Self {
            config: RequestIdConfig {
                enabled: true,
                header_name: default_header_name(),
            }
        }
    }
    
    pub fn with_config(config: RequestIdConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Plugin for RequestId {
    fn name(&self) -> &'static str {
        "request_id"
    }
    
    /// Generate a unique request ID and add it to the context
    async fn handle_request(
        &self,
        _step: PluginStep,
        _session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // Only generate request ID if the plugin is enabled
        if !self.config.enabled {
            return Ok((false, None));
        }

        // Generate a UUID for the request ID if it's empty
        if ctx.request_id.is_empty() {
            let uuid = Uuid::new_v4().to_string();
            ctx.request_id = uuid.clone();
            
            // Also store in extensions for tests to check
            ctx.extensions.insert(Cow::Borrowed("request_id_header"), uuid);
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
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}