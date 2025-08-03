use std::time::SystemTime;
use std::{borrow::Cow, collections::HashMap};

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use cookie::Cookie;
use http::StatusCode;
use once_cell::sync::Lazy;
use pingora::http::ResponseHeader;
use pingora::proxy::Session;
use serde::{Deserialize, Serialize};

use provider::{OauthType, OauthUser, Provider};

use crate::proxy_server::https_proxy::RouterContext;
use crate::proxy_server::HttpResponse;
use crate::plugins::core::{Plugin, PluginError, PluginStep, PluginMetadata, PluginType};
use tracing::{debug, info};

use super::get_required_config;
use super::jwt;

// New providers can be added here
mod github;
mod workos;
//
mod provider;
mod secure_cookie;
mod shared;

/// The HTTP client used to make requests to the Oauth provider.
/// Lazy loaded to avoid creating a new client for each request.
#[allow(dead_code)]
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

// Shared state for Oauth2 flows (should be cleaned up after fetching for the first time)
// TODO find a way to clean up/expire the state
const COOKIE_NAME: &str = "__Secure_Auth_PRK_JWT";

fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Configuration for OAuth2 plugin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuth2Config {
    /// Provider type (github, workos, etc.)
    pub provider: String,
    
    /// Client ID from OAuth provider
    pub client_id: String,
    
    /// Client secret from OAuth provider 
    pub client_secret: String,
    
    /// JWT secret for signing cookies
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    
    /// Redirect URL after successful authentication
    #[serde(default)]
    pub redirect_url: Option<String>,
    
    /// Additional validations
    #[serde(default)]
    pub validations: Option<serde_json::Value>,
    
    /// Whether to enable this plugin
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_jwt_secret() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_true() -> bool {
    true
}

/// The Oauth2 plugin
/// This plugin is responsible for handling Oauth authentication and authorization
/// It can be used to authenticate users against various Oauth providers
/// and authorize them to access specific resources or perform certain actions
/// based on their authorization level.
#[derive(Debug)]
pub struct Oauth2 {
    config: Option<OAuth2Config>,
    short_crypt: short_crypt::ShortCrypt,
}

impl Oauth2 {
    pub fn new() -> Self {
        // Generates in-memory secret for oauth2 states
        let short_crypt = short_crypt::ShortCrypt::new(uuid::Uuid::new_v4().to_string());

        Self { 
            config: None,
            short_crypt 
        }
    }
    
    pub fn with_config(config: OAuth2Config) -> Self {
        let short_crypt = short_crypt::ShortCrypt::new(uuid::Uuid::new_v4().to_string());
        
        Self {
            config: Some(config),
            short_crypt
        }
    }

    /// Checks if the user is authorized to access the protected Oauth2 resource
    /// This is part of the validation object in the oauth2 configuration.
    fn is_authorized(user: &OauthUser, validations: Option<&serde_json::Value>) -> bool {
        shared::validate_user_from_provider(user, validations)
    }

    /// Redirects the user to the Oauth provider to authenticate.
    async fn redirect_to_oauth_callback(
        &self,
        session: &mut Session,
        oauth_provider: &Provider,
    ) -> Result<bool> {
        let current_address = session.req_header().uri.to_string();
        // let host = session.req_header().uri.host().unwrap_or_default();

        let state = format!("{};{}", get_current_timestamp(), current_address);
        let state = self.short_crypt.encrypt_to_url_component(&state);

        let mut res_headers =
            ResponseHeader::build_no_case(StatusCode::TEMPORARY_REDIRECT, Some(1))?;

        // Removes any cookie to prevent the user from being redirected
        // to the Oauth provider over and over
        // let removed_cookie = remove_secure_cookie(host);
        // res_headers.append_header(http::header::SET_COOKIE, removed_cookie.to_string())?;

        res_headers.append_header(
            http::header::LOCATION,
            oauth_provider.get_oauth_callback_url(&state),
        )?;
        res_headers.append_header(
            http::header::CACHE_CONTROL,
            "no-store, no-cache, must-revalidate, max-age=0",
        )?;

        // Store the current path in the state
        session
            .write_response_header(Box::new(res_headers), true)
            .await?;

        // Finish the request, we don't want to continue processing
        // and the user has been redirected
        Ok(true)
    }

    /// Ends the Oauth2 flow and returns HTTP unauthorized if errors occur during the Oauth process
    async fn unauthorized_response(&self, session: &mut Session) -> Result<bool> {
        let res_headers = ResponseHeader::build_no_case(StatusCode::UNAUTHORIZED, Some(1))?;

        // Store the current path in the stat
        session
            .write_response_header(Box::new(res_headers), true)
            .await?;

        // Finish the request, we don't want to continue processing
        // and the user has been redirected
        Ok(true)
    }

    /// Validates a given cookie and returns true if
    /// the user is authorized
    async fn validate_cookie(
        &self,
        session: &mut Session,
        jwt_secret: &str,
        validations: Option<&serde_json::Value>,
    ) -> Result<bool> {
        let cookie_header = session.req_header().headers.get_all("cookie");

        let mut secure_jwt: Option<Cookie> = None;

        for cookie in cookie_header {
            let cookie_str = cookie.to_str().unwrap();

            let Ok(ck) = Cookie::parse(cookie_str) else {
                continue;
            };

            if ck.name() == COOKIE_NAME {
                secure_jwt = Some(ck);
            }
        }

        if secure_jwt.is_none() {
            return Ok(false); // will redirect to oauth callback
        }

        let decoded = jwt::decode_jwt(secure_jwt.unwrap().value(), jwt_secret.as_bytes());

        // Token expired or another err
        if decoded.is_err() {
            return Ok(false); // will redirect to oauth callback
        }

        if !Self::is_authorized(&decoded?.into(), validations) {
            return self.unauthorized_response(session).await;
        }

        Ok(true)
    }

    fn parse_provider(
        plugin_config: &HashMap<Cow<'static, str>, serde_json::Value>,
    ) -> Result<OauthType> {
        let provider = plugin_config
            .get("provider")
            .and_then(|v| v.as_str())
            .ok_or(anyhow!("Missing or invalid provider"))?;

        match provider {
            "github" => Ok(OauthType::Github),
            "workos" => Ok(OauthType::Workos),
            _ => bail!("Provider not found in the plugin configuration"),
        }
    }
    
    /// Process the OAuth2 authentication request
    async fn process_oauth(&self, session: &mut Session, plugin_config: &HashMap<Cow<'static, str>, serde_json::Value>) -> Result<bool> {
        let provider_type = Self::parse_provider(plugin_config)?;
        let client_id = get_required_config(plugin_config, "client_id")?;
        let client_secret = get_required_config(plugin_config, "client_secret")?;
        let jwt_secret = plugin_config
            .get("jwt_secret")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let valdiations = plugin_config.get("validations");

        let is_callback = session
            .req_header()
            .uri
            .query()
            .map_or(false, |q| q.contains("code="));

        // This is a callback from the Oauth provider
        if is_callback {
            // process the callback
            // TO BE IMPLEMENTED
            return Ok(false);
        }

        // Check if the user already has a valid cookie
        let is_cookie_ok = self
            .validate_cookie(session, &jwt_secret, valdiations)
            .await?;

        // User is authenticated with cookie
        if is_cookie_ok {
            return Ok(false);
        }

        // No valid cookie, redirect to Oauth provider
        let oauth_provider = match provider_type {
            OauthType::Github => github::GithubOauthService::new(&client_id, &client_secret),
            OauthType::Workos => workos::WorkosOauthService::new(&client_id, &client_secret),
        };

        // No cookie or the cookie is invalid
        // Redirect to Oauth callback
        self.redirect_to_oauth_callback(session, &oauth_provider).await
    }

    #[allow(dead_code)]
    async fn process_response(
        &self,
        _ctx: &mut RouterContext,
        _step: PluginStep,
        _session: &mut Session,
        _upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        // ... existing code ...
        Ok(false)
    }
}

#[async_trait]
impl Plugin for Oauth2 {
    fn name(&self) -> &'static str {
        "oauth2"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 90, // OAuth should run early, but after request ID
            plugin_type: PluginType::Native,
            description: "OAuth2 authentication plugin supporting GitHub, WorkOS and other providers".to_string(),
            author: "Proksi Team".to_string(),
            homepage: Some("https://github.com/luizfonseca/proksi".to_string()),
        }
    }
    
    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        _ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // Only process in the EarlyRequest step
        if step != PluginStep::EarlyRequest {
            return Ok((false, None));
        }
        
        // If no config is provided, or plugin is disabled, skip
        let Some(config) = &self.config else {
            debug!("OAuth2 plugin has no configuration, skipping");
            return Ok((false, None));
        };

        if !config.enabled {
            debug!("OAuth2 plugin is disabled, skipping");
            return Ok((false, None));
        }

        debug!("Processing OAuth2 authentication for provider: {}", config.provider);
        
        // Convert config to HashMap for compatibility with existing code
        let mut plugin_config = HashMap::new();
        plugin_config.insert(Cow::Borrowed("provider"), serde_json::to_value(&config.provider).unwrap());
        plugin_config.insert(Cow::Borrowed("client_id"), serde_json::to_value(&config.client_id).unwrap());
        plugin_config.insert(Cow::Borrowed("client_secret"), serde_json::to_value(&config.client_secret).unwrap());
        plugin_config.insert(Cow::Borrowed("jwt_secret"), serde_json::to_value(&config.jwt_secret).unwrap());
        
        if let Some(validations) = &config.validations {
            plugin_config.insert(Cow::Borrowed("validations"), validations.clone());
        }
        
        if let Some(redirect_url) = &config.redirect_url {
            plugin_config.insert(Cow::Borrowed("redirect_url"), serde_json::to_value(redirect_url).unwrap());
        }
        
        // Process the OAuth request
        match self.process_oauth(session, &plugin_config).await {
            Ok(true) => Ok((true, None)), // OAuth processing handled the request
            Ok(false) => Ok((false, None)), // Continue with normal request processing
            Err(e) => {
                // In case of error, return unauthorized
                let response = HttpResponse {
                    status_code: StatusCode::UNAUTHORIZED,
                    headers: http::HeaderMap::new(),
                    body: format!("OAuth error: {}", e).into(),
                };
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
        // No response handling needed
        Ok(false)
    }
    
    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting OAuth2 plugin");
        if let Some(config) = &self.config {
            if config.enabled {
                info!("OAuth2 plugin started for provider: {}", config.provider);
            } else {
                info!("OAuth2 plugin is disabled");
            }
        } else {
            info!("OAuth2 plugin has no configuration");
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping OAuth2 plugin");
        Ok(())
    }
}
