use std::time::SystemTime;
use std::{borrow::Cow, collections::HashMap};

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use cookie::Cookie;
use http::StatusCode;
use once_cell::sync::Lazy;
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::proxy::Session;

use provider::{OauthType, OauthUser, Provider};

use crate::{config::RoutePlugin, proxy_server::https_proxy::RouterContext};

use super::{get_required_config, jwt};

// New providers can be added here
mod github;
mod workos;
//
mod provider;
mod secure_cookie;
mod shared;

/// The HTTP client used to make requests to the Oauth provider.
/// Lazy loaded to avoid creating a new client for each request.
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

/// The Oauth2 plugin
/// This plugin is responsible for handling Oauth authentication and authorization
/// It can be used to authenticate users against various Oauth providers
/// and authorize them to access specific resources or perform certain actions
/// based on their authorization level.
pub struct Oauth2 {
    short_crypt: short_crypt::ShortCrypt,
}

impl Oauth2 {
    pub fn new() -> Self {
        // Generates in-memory secret for oauth2 states
        let short_crypt = short_crypt::ShortCrypt::new(uuid::Uuid::new_v4().to_string());

        Self { short_crypt }
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
}

/* // Commented out outdated implementation
#[async_trait]
impl MiddlewarePlugin for Oauth2 {
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut RouterContext,
        plugin: &RoutePlugin,
    ) -> anyhow::Result<bool> {
        // ... implementation ...
    }

    async fn upstream_request_filter(
        &self,
        _: &mut Session,
        _: &mut RequestHeader,
        _: &mut RouterContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn response_filter(
        &self,
        _: &mut Session,
        _: &mut RouterContext,
        _: &RoutePlugin,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn upstream_response_filter(
        &self,
        _: &mut Session,
        _: &mut ResponseHeader,
        _: &mut RouterContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
*/
