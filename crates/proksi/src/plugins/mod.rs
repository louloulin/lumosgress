use std::{borrow::Cow, collections::HashMap};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use basic_auth::BasicAuth;
use oauth2::Oauth2;
use once_cell::sync::Lazy;
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::proxy::Session;
use request_id::RequestId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{config::RoutePlugin, proxy_server::https_proxy::RouterContext};

// 新的插件系统模块
pub mod core;
mod core_test;
pub mod executor;

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
pub mod api_server;

pub mod tenant;
pub mod compliance;
pub mod manager;

use llm_router::LlmRouter;
use prompt_transform::PromptTransformer;
use ai_security::AiSecurity;
use llm_aggregator::LlmAggregator;
use vector_db::VectorDb;
use ai_analytics::AiAnalytics;
use ai_request_builder::AiRequestBuilder;
use prompt_debugger::PromptDebugger;
use performance_analyzer::PerformanceAnalyzer;

// 重新导出新的插件接口
pub use core::{Plugin as NewPlugin, PluginType, PluginStep, PluginError as NewPluginError, 
               PluginFactory, PluginMetadata, PluginRegistry};
pub use executor::{execute_plugins, initialize_plugins, shutdown_plugins};

pub(crate) struct ProxyPlugins {
    pub basic_auth: Lazy<BasicAuth>,
    pub oauth2: Lazy<Oauth2>,
    pub request_id: Lazy<request_id::RequestId>,
    
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
    request_id: Lazy::new(request_id::RequestId::new),
    
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
