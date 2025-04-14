use std::sync::Arc;
use std::fmt;

use async_trait::async_trait;
use http::{header, StatusCode};
use openssl::base64;
use pingora::{
    http::{RequestHeader, ResponseHeader},
    proxy::Session,
};
use crate::proxy_server::HttpResponse;
use serde::Deserialize;

use crate::{proxy_server::https_proxy::RouterContext};
use super::core::{Plugin, PluginStep, PluginError, PluginMetadata, PluginType};

#[derive(Debug, Clone, Deserialize)]
pub struct BasicAuthConfig {
    user: String,
    pass: String,
}

#[derive(Clone)]
pub struct BasicAuth {
    config: BasicAuthConfig,
}

impl fmt::Debug for BasicAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicAuth")
         .field("config.user", &self.config.user)
         .field("config.pass", &"<hidden>")
         .finish()
    }
}

impl BasicAuth {
    pub fn new(config: BasicAuthConfig) -> Self {
        Self { config }
    }

    fn respond_with_authenticate(host: &str) -> Result<HttpResponse, PluginError> {
        let mut pingora_res_headers = ResponseHeader::build(StatusCode::UNAUTHORIZED, None)
            .map_err(|e| PluginError::ConfigError(e.to_string()))?;
        let realm = format!("Basic realm=\"{}\", charset=\"UTF-8\"", host);
        pingora_res_headers
            .insert_header(header::WWW_AUTHENTICATE, &realm)
            .map_err(|e| PluginError::ConfigError(e.to_string()))?;

        let mut http_headers = http::HeaderMap::new();
        for (name, value) in pingora_res_headers.headers.iter() {
            if let Ok(header_value) = http::HeaderValue::from_bytes(value.as_bytes()) {
                http_headers.append(name.clone(), header_value);
            } else {
                return Err(PluginError::ConfigError(format!(
                    "Failed to convert header value for key: {}",
                    name
                )));
            }
        }

        let response = HttpResponse {
            status_code: StatusCode::UNAUTHORIZED,
            headers: http_headers,
            body: bytes::Bytes::new(),
        };

        Ok(response)
    }

    fn validate_auth_header(&self, auth_header: &str) -> Result<bool, PluginError> {
        let encoded = auth_header.trim_start_matches("Basic ");
        let decoded_bytes = base64::decode_block(encoded)
            .map_err(|e| PluginError::ConfigError(format!("Base64 decode error: {}", e)))?;
        let decoded = String::from_utf8(decoded_bytes)
            .map_err(|e| PluginError::ConfigError(format!("UTF8 decode error: {}", e)))?;
        let (auth_user, auth_pass) = decoded.split_once(':').unwrap_or_default();
        Ok(auth_user == self.config.user && auth_pass == self.config.pass)
    }
}

#[async_trait]
impl Plugin for BasicAuth {
    fn name(&self) -> &'static str {
        "basic_auth"
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>), anyhow::Error> {
        if step != PluginStep::Request {
            return Ok((false, None));
        }

        let auth_header_opt = session.req_header().headers.get("authorization");

        match auth_header_opt {
            Some(auth_header) => {
                let auth_str = auth_header.to_str()
                    .map_err(|e| PluginError::ConfigError(format!("Invalid auth header: {}", e)))?;

                if !auth_str.starts_with("Basic ")
                    || !self.validate_auth_header(auth_str)?
                {
                    let response = Self::respond_with_authenticate(&ctx.host)?;
                    Ok((true, Some(response)))
                } else {
                    Ok((false, None))
                }
            }
            None => {
                let response = Self::respond_with_authenticate(&ctx.host)?;
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
    ) -> Result<bool, anyhow::Error> {
        Ok(false)
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
