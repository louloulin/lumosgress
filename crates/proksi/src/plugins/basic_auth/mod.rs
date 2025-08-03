use std::fmt;

use anyhow::Result;
use async_trait::async_trait;
use http::{header, StatusCode};
use openssl::base64;
use pingora::{
    http::ResponseHeader,
    proxy::Session,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::proxy_server::HttpResponse;
use crate::proxy_server::https_proxy::RouterContext;
use super::core::{Plugin, PluginStep, PluginError, PluginMetadata, PluginType};

/// Basic authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    /// Username for authentication
    pub user: String,
    /// Password for authentication (will be hidden in debug output)
    pub pass: String,
    /// Enable/disable the plugin
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Custom realm name
    #[serde(default = "default_realm")]
    pub realm: String,
}

fn default_enabled() -> bool {
    true
}

fn default_realm() -> String {
    "Proksi Gateway".to_string()
}

impl Default for BasicAuthConfig {
    fn default() -> Self {
        Self {
            user: "admin".to_string(),
            pass: "password".to_string(),
            enabled: true,
            realm: default_realm(),
        }
    }
}

/// Basic authentication plugin
#[derive(Clone)]
pub struct BasicAuth {
    config: BasicAuthConfig,
}

impl fmt::Debug for BasicAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicAuth")
         .field("enabled", &self.config.enabled)
         .field("user", &self.config.user)
         .field("realm", &self.config.realm)
         .field("pass", &"<hidden>")
         .finish()
    }
}

impl BasicAuth {
    /// Create a new BasicAuth plugin instance
    pub fn new(config: BasicAuthConfig) -> Self {
        info!("Initializing BasicAuth plugin for user: {}", config.user);
        Self { config }
    }

    /// Create a new BasicAuth plugin with default configuration
    pub fn default() -> Self {
        Self::new(BasicAuthConfig::default())
    }

    /// Generate an authentication challenge response
    fn respond_with_authenticate(&self, host: &str) -> Result<HttpResponse, PluginError> {
        debug!("Generating authentication challenge for host: {}", host);

        let mut pingora_res_headers = ResponseHeader::build(StatusCode::UNAUTHORIZED, None)
            .map_err(|e| {
                error!("Failed to build response header: {}", e);
                PluginError::ConfigError(format!("Failed to build response header: {}", e))
            })?;

        let realm = format!("Basic realm=\"{}\", charset=\"UTF-8\"", self.config.realm);
        pingora_res_headers
            .insert_header(header::WWW_AUTHENTICATE, &realm)
            .map_err(|e| {
                error!("Failed to insert WWW-Authenticate header: {}", e);
                PluginError::ConfigError(format!("Failed to insert WWW-Authenticate header: {}", e))
            })?;

        let mut http_headers = http::HeaderMap::new();
        for (name, value) in pingora_res_headers.headers.iter() {
            if let Ok(header_value) = http::HeaderValue::from_bytes(value.as_bytes()) {
                http_headers.append(name.clone(), header_value);
            } else {
                error!("Failed to convert header value for key: {}", name);
                return Err(PluginError::ConfigError(format!(
                    "Failed to convert header value for key: {}",
                    name
                )));
            }
        }

        let response = HttpResponse {
            status_code: StatusCode::UNAUTHORIZED,
            headers: http_headers,
            body: bytes::Bytes::from("Authentication required"),
        };

        debug!("Authentication challenge response created");
        Ok(response)
    }

    /// Validate the provided authorization header
    fn validate_auth_header(&self, auth_header: &str) -> Result<bool, PluginError> {
        debug!("Validating authorization header");

        if !auth_header.starts_with("Basic ") {
            warn!("Invalid authorization header format");
            return Ok(false);
        }

        let encoded = auth_header.trim_start_matches("Basic ");
        let decoded_bytes = base64::decode_block(encoded)
            .map_err(|e| {
                warn!("Base64 decode error: {}", e);
                PluginError::ConfigError(format!("Base64 decode error: {}", e))
            })?;

        let decoded = String::from_utf8(decoded_bytes)
            .map_err(|e| {
                warn!("UTF8 decode error: {}", e);
                PluginError::ConfigError(format!("UTF8 decode error: {}", e))
            })?;

        let (auth_user, auth_pass) = decoded.split_once(':').unwrap_or(("", ""));

        let is_valid = auth_user == self.config.user && auth_pass == self.config.pass;

        if is_valid {
            debug!("Authentication successful for user: {}", auth_user);
        } else {
            warn!("Authentication failed for user: {}", auth_user);
        }

        Ok(is_valid)
    }
}

#[async_trait]
impl Plugin for BasicAuth {
    fn name(&self) -> &'static str {
        "basic_auth"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 100, // Authentication should run early
            plugin_type: PluginType::Native,
            description: "HTTP Basic Authentication plugin for securing endpoints".to_string(),
            author: "Proksi Team".to_string(),
            homepage: Some("https://github.com/luizfonseca/proksi".to_string()),
        }
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // Only process during the Request step
        if step != PluginStep::Request {
            return Ok((false, None));
        }

        // Check if plugin is enabled
        if !self.config.enabled {
            debug!("BasicAuth plugin is disabled, skipping");
            return Ok((false, None));
        }

        debug!("Processing BasicAuth for request: {}", ctx.request_id);

        let auth_header_opt = session.req_header().headers.get("authorization");

        match auth_header_opt {
            Some(auth_header) => {
                let auth_str = auth_header.to_str()
                    .map_err(|e| PluginError::ConfigError(format!("Invalid auth header: {}", e)))?;

                match self.validate_auth_header(auth_str) {
                    Ok(true) => {
                        debug!("Authentication successful, allowing request");
                        Ok((false, None))
                    }
                    Ok(false) => {
                        info!("Authentication failed, returning challenge");
                        let host = session.req_header().uri.host().unwrap_or("localhost");
                        let response = self.respond_with_authenticate(host)?;
                        Ok((true, Some(response)))
                    }
                    Err(e) => {
                        error!("Authentication error: {}", e);
                        Err(anyhow::anyhow!("Authentication error: {}", e))
                    }
                }
            }
            None => {
                info!("No authorization header found, returning challenge");
                let host = session.req_header().uri.host().unwrap_or("localhost");
                let response = self.respond_with_authenticate(host)?;
                Ok((true, Some(response)))
            }
        }
    }

    async fn handle_response(
        &self,
        _step: PluginStep,
        _session: &mut Session,
        _ctx: &mut RouterContext,
        _upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        // BasicAuth doesn't modify responses
        Ok(false)
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting BasicAuth plugin");
        if !self.config.enabled {
            info!("BasicAuth plugin is disabled");
        } else {
            info!("BasicAuth plugin started for user: {}", self.config.user);
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping BasicAuth plugin");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth_config_default() {
        let config = BasicAuthConfig::default();
        assert_eq!(config.user, "admin");
        assert_eq!(config.pass, "password");
        assert!(config.enabled);
        assert_eq!(config.realm, "Proksi Gateway");
    }

    #[test]
    fn test_basic_auth_creation() {
        let config = BasicAuthConfig {
            user: "testuser".to_string(),
            pass: "testpass".to_string(),
            enabled: true,
            realm: "Test Realm".to_string(),
        };
        let auth = BasicAuth::new(config.clone());
        assert_eq!(auth.config.user, "testuser");
        assert_eq!(auth.config.realm, "Test Realm");
    }

    #[test]
    fn test_validate_auth_header_success() {
        let config = BasicAuthConfig {
            user: "admin".to_string(),
            pass: "password".to_string(),
            enabled: true,
            realm: "Test".to_string(),
        };
        let auth = BasicAuth::new(config);

        // "admin:password" in base64 is "YWRtaW46cGFzc3dvcmQ="
        let result = auth.validate_auth_header("Basic YWRtaW46cGFzc3dvcmQ=");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_validate_auth_header_failure() {
        let config = BasicAuthConfig {
            user: "admin".to_string(),
            pass: "password".to_string(),
            enabled: true,
            realm: "Test".to_string(),
        };
        let auth = BasicAuth::new(config);

        // "admin:wrongpass" in base64 is "YWRtaW46d3JvbmdwYXNz"
        let result = auth.validate_auth_header("Basic YWRtaW46d3JvbmdwYXNz");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_validate_auth_header_invalid_format() {
        let config = BasicAuthConfig::default();
        let auth = BasicAuth::new(config);

        let result = auth.validate_auth_header("Bearer token123");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_plugin_metadata() {
        let auth = BasicAuth::default();
        let metadata = auth.metadata();
        assert_eq!(metadata.name, "basic_auth");
        assert_eq!(metadata.priority, 100);
        assert_eq!(metadata.plugin_type, PluginType::Native);
        assert!(!metadata.description.is_empty());
    }
}
