use std::borrow::Cow;
use std::collections::HashMap;

use anyhow::{anyhow, Result};
use basic_auth::{BasicAuth, BasicAuthConfig};
use oauth2::Oauth2;
use once_cell::sync::Lazy;
use request_id::RequestId;

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
pub mod async_api;
pub mod prompt_debugger;
pub mod performance_analyzer;
pub mod api_server;

pub mod tenant;
pub mod compliance;
pub mod manager;

// 重新导出新的插件接口
pub use core::{Plugin, PluginType, PluginStep, PluginError, 
               PluginFactory, PluginMetadata, PluginRegistry};
pub use executor::{execute_plugins, initialize_plugins, shutdown_plugins};

pub use tenant::TenantPlugin;
pub use compliance::CompliancePlugin;
pub use api_server::ApiServerPlugin;

pub struct ProxyPlugins {
    pub basic_auth: Lazy<BasicAuth>,
    pub oauth2: Lazy<Oauth2>,
    pub request_id: Lazy<request_id::RequestId>,
    
    pub llm_router: Lazy<llm_router::LlmRouter>,
    pub prompt_transform: Lazy<prompt_transform::PromptTransformer>,
    pub ai_security: Lazy<ai_security::AiSecurity>,
    pub llm_aggregator: Lazy<llm_aggregator::LlmAggregator>,
    pub vector_db: Lazy<vector_db::VectorDb>,
    pub ai_analytics: Lazy<ai_analytics::AiAnalytics>,
    pub ai_request_builder: Lazy<ai_request_builder::AiRequestBuilder>,
    pub async_api: Lazy<async_api::AsyncApiPlugin>,
    pub prompt_debugger: Lazy<prompt_debugger::PromptDebugger>,
    pub performance_analyzer: Lazy<performance_analyzer::PerformanceAnalyzer>,
}

/// Static plugin registry (plugins that don't generate a new instance for each request)
pub static PLUGINS: Lazy<ProxyPlugins> = Lazy::new(|| ProxyPlugins {
    basic_auth: Lazy::new(|| BasicAuth::new(BasicAuthConfig {
        user: "admin".to_string(),
        pass: "password".to_string(),
    })),
    oauth2: Lazy::new(Oauth2::new),
    request_id: Lazy::new(RequestId::new),
    
    llm_router: Lazy::new(llm_router::LlmRouter::new),
    prompt_transform: Lazy::new(prompt_transform::PromptTransformer::new),
    ai_security: Lazy::new(ai_security::AiSecurity::new),
    llm_aggregator: Lazy::new(llm_aggregator::LlmAggregator::new),
    vector_db: Lazy::new(vector_db::VectorDb::new),
    ai_analytics: Lazy::new(ai_analytics::AiAnalytics::new),
    ai_request_builder: Lazy::new(ai_request_builder::AiRequestBuilder::new),
    async_api: Lazy::new(async_api::AsyncApiPlugin::new),
    prompt_debugger: Lazy::new(prompt_debugger::PromptDebugger::new),
    performance_analyzer: Lazy::new(performance_analyzer::PerformanceAnalyzer::new),
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
