use std::{borrow::Cow, collections::HashMap};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use basic_auth::BasicAuth;
use oauth2::Oauth2;
use once_cell::sync::Lazy;
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::proxy::Session;
use request_id::RequestId;

use crate::{config::RoutePlugin, proxy_server::https_proxy::RouterContext};

pub mod basic_auth;
pub mod jwt;
pub mod oauth2;
pub mod request_id;

pub mod llm_router;
pub mod prompt_transform;
pub mod ai_security;
pub mod llm_aggregator;
pub mod vector_db;
pub mod ai_analytics;
pub mod ai_request_builder;
pub mod prompt_debugger;
pub mod performance_analyzer;

use llm_router::LlmRouter;
use prompt_transform::PromptTransformer;
use ai_security::AiSecurity;
use llm_aggregator::LlmAggregator;
use vector_db::VectorDb;
use ai_analytics::AiAnalytics;
use ai_request_builder::AiRequestBuilder;
use prompt_debugger::PromptDebugger;
use performance_analyzer::PerformanceAnalyzer;

pub(crate) struct ProxyPlugins {
    pub basic_auth: Lazy<BasicAuth>,
    pub oauth2: Lazy<Oauth2>,
    pub request_id: Lazy<RequestId>,
    
    pub llm_router: Lazy<LlmRouter>,
    pub prompt_transform: Lazy<PromptTransformer>,
    pub ai_security: Lazy<AiSecurity>,
    pub llm_aggregator: Lazy<LlmAggregator>,
    pub vector_db: Lazy<VectorDb>,
    pub ai_analytics: Lazy<AiAnalytics>,
    pub ai_request_builder: Lazy<AiRequestBuilder>,
    pub prompt_debugger: Lazy<PromptDebugger>,
    pub performance_analyzer: Lazy<PerformanceAnalyzer>,
}

/// Static plugin registry (plugins that don't generate a new instance for each request)
pub static PLUGINS: Lazy<ProxyPlugins> = Lazy::new(|| ProxyPlugins {
    basic_auth: Lazy::new(BasicAuth::new),
    oauth2: Lazy::new(Oauth2::new),
    request_id: Lazy::new(RequestId::new),
    
    llm_router: Lazy::new(LlmRouter::new),
    prompt_transform: Lazy::new(PromptTransformer::new),
    ai_security: Lazy::new(AiSecurity::new),
    llm_aggregator: Lazy::new(LlmAggregator::new),
    vector_db: Lazy::new(VectorDb::new),
    ai_analytics: Lazy::new(AiAnalytics::new),
    ai_request_builder: Lazy::new(AiRequestBuilder::new),
    prompt_debugger: Lazy::new(PromptDebugger::new),
    performance_analyzer: Lazy::new(PerformanceAnalyzer::new),
});

/// Get a required configuration value from a plugin config
fn get_required_config(
    plugin_config: &HashMap<Cow<'static, str>, serde_json::Value>,
    key: &str,
) -> Result<String> {
    plugin_config
        .get(key)
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("Missing or invalid {}", key))
}

#[async_trait]
pub trait MiddlewarePlugin {
    /// Create a new state for the middleware
    /// Filter requests based on the middleware's logic
    /// Return false if the request should be allowed to pass through and was not handled
    /// Return true if the request was already handled
    async fn request_filter(
        &self,
        session: &mut Session,
        state: &mut RouterContext,
        config: &RoutePlugin,
    ) -> Result<bool>;

    /// Modify the request before it is sent to the upstream
    ///
    /// Unlike [Self::request_filter()], this filter allows to
    /// change the request headers before it hits your server.
    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        state: &mut RouterContext,
    ) -> Result<()>;

    /// Filter responses (from upstream) based on the middleware's logic
    /// Return false if the request should be allowed to pass through and was not handled
    /// Return true if the request was already handled
    async fn response_filter(
        &self,
        session: &mut Session,
        state: &mut RouterContext,
        config: &RoutePlugin,
    ) -> Result<bool>;

    fn upstream_response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        state: &mut RouterContext,
    ) -> Result<()>;
}
