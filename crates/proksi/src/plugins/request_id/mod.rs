use std::borrow::Cow;

use anyhow::{Result, Error};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use pingora::proxy::Session;
use pingora::http::{RequestHeader, ResponseHeader};

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
    
    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // Only process in the Request step
        if step != PluginStep::Request || !self.config.enabled {
            return Ok((false, None));
        }
        
        // Generate a new request ID using UUID v4
        let request_id = Uuid::new_v4().to_string();
        
        // Store in the context for later use
        ctx.request_id = request_id.clone();
        
        // Also store in extensions for backward compatibility
        ctx.extensions
            .insert(Cow::Borrowed("request_id_header"), request_id);
            
        Ok((false, None))
    }
    
    async fn handle_response(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
        upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        if step != PluginStep::Response {
            return Ok(false);
        }
        if let Some(request_id_val) = ctx.plugins_data.get("request_id") {
            if let Some(request_id_str) = request_id_val.as_str() {
                // Fix lifetime error: Clone both the header name and request_id to own them
                let header_name = self.config.header_name.clone();
                upstream_response.insert_header(header_name, request_id_str.to_string())?;
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
