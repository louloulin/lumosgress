use std::{borrow::Cow, collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use http::HeaderValue;
use once_cell::sync::Lazy;
use pingora::{
    http::{RequestHeader, ResponseHeader},
    proxy::Session,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    config::RoutePlugin,
    plugins::get_required_config,
    proxy_server::https_proxy::RouterContext,
};

use super::MiddlewarePlugin;

// 支持的LLM提供商
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    GoogleVertexAI,
    AzureOpenAI,
    Custom,
}

impl From<&str> for LlmProvider {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => LlmProvider::OpenAI,
            "anthropic" => LlmProvider::Anthropic,
            "google" | "vertex" | "vertexai" => LlmProvider::GoogleVertexAI,
            "azure" | "azureopenai" => LlmProvider::AzureOpenAI,
            _ => LlmProvider::Custom,
        }
    }
}

// LLM路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRouterConfig {
    pub default_provider: LlmProvider,
    pub providers: HashMap<String, LlmProviderConfig>,
    pub semantic_routing_enabled: bool,
}

// 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProviderConfig {
    pub endpoint: String,
    pub api_key_env: Option<String>,
    pub weight: Option<u8>,
    pub models: Vec<String>,
}

pub struct LlmRouter {
    pub config: Arc<HashMap<String, LlmRouterConfig>>,
}

impl LlmRouter {
    pub fn new() -> Self {
        Self {
            config: Arc::new(HashMap::new()),
        }
    }

    // 根据请求内容确定应该路由到哪个LLM提供商
    async fn determine_provider(&self, session: &mut Session, config_name: &str) -> Result<Option<(String, String)>> {
        let configs = self.config.clone();
        
        if let Some(router_config) = configs.get(config_name) {
            // 从请求体中提取提示内容进行语义路由
            if router_config.semantic_routing_enabled {
                // 这里只是简单实现，实际需要根据请求体和提示内容进行分析
                if let Some(content_type) = session.req_header().headers.get("content-type") {
                    if content_type.to_str().unwrap_or("").contains("application/json") {
                        // 实际实现中应该解析请求体，分析提示内容，然后决定哪个提供商最适合
                        
                        // 简单检查请求体大小以决定路由
                        if let Some(content_length) = session.req_header().headers.get("content-length") {
                            if let Ok(length) = content_length.to_str().unwrap_or("0").parse::<usize>() {
                                // 假设：较大的请求更适合某些特定模型
                                if length > 1000 {
                                    // 找到支持长文本的提供商
                                    for (name, provider) in &router_config.providers {
                                        // 根据实际场景完善此逻辑
                                        if name.contains("anthropic") || name.contains("claude") {
                                            if let Some(endpoint) = provider.endpoint.split('/').last() {
                                                return Ok(Some((name.clone(), endpoint.to_string())));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // 默认情况使用默认提供商
            let default = &router_config.default_provider;
            for (name, provider) in &router_config.providers {
                match default {
                    LlmProvider::OpenAI if name.contains("openai") => {
                        if let Some(endpoint) = provider.endpoint.split('/').last() {
                            return Ok(Some((name.clone(), endpoint.to_string())));
                        }
                    }
                    LlmProvider::Anthropic if name.contains("anthropic") => {
                        if let Some(endpoint) = provider.endpoint.split('/').last() {
                            return Ok(Some((name.clone(), endpoint.to_string())));
                        }
                    }
                    LlmProvider::GoogleVertexAI if name.contains("google") => {
                        if let Some(endpoint) = provider.endpoint.split('/').last() {
                            return Ok(Some((name.clone(), endpoint.to_string())));
                        }
                    }
                    LlmProvider::AzureOpenAI if name.contains("azure") => {
                        if let Some(endpoint) = provider.endpoint.split('/').last() {
                            return Ok(Some((name.clone(), endpoint.to_string())));
                        }
                    }
                    _ => {}
                }
            }
            
            // 如果找不到匹配的提供商，使用第一个
            if let Some((name, provider)) = router_config.providers.iter().next() {
                if let Some(endpoint) = provider.endpoint.split('/').last() {
                    return Ok(Some((name.clone(), endpoint.to_string())));
                }
            }
        }
        
        Ok(None)
    }
}

// 实现中间件插件 trait
#[async_trait]
impl MiddlewarePlugin for LlmRouter {
    async fn request_filter(
        &self,
        session: &mut Session,
        state: &mut RouterContext,
        config: &RoutePlugin,
    ) -> Result<bool> {
        // 获取插件配置
        let config_data = config.config.as_ref().ok_or_else(|| anyhow!("LLM Router plugin requires configuration"))?;
        
        // 获取配置名
        let config_name = get_required_config(config_data, "config_name").unwrap_or_else(|_| "default".to_string());
        
        // 确定要使用的LLM提供商
        if let Some((provider_name, endpoint)) = self.determine_provider(session, &config_name).await? {
            // 将提供商信息存储在上下文中，以便后续使用
            state.extensions.insert(Cow::Borrowed("llm_provider"), provider_name.clone());
            state.extensions.insert(Cow::Borrowed("llm_endpoint"), endpoint.clone());
            
            info!(
                provider = provider_name,
                endpoint = endpoint,
                "LLM request routed"
            );
        } else {
            warn!("No suitable LLM provider found for request");
        }
        
        // 继续处理请求
        Ok(false)
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        state: &mut RouterContext,
    ) -> Result<()> {
        // 如果之前确定了LLM提供商，修改请求以使用该提供商
        if let Some(provider) = state.extensions.get(&Cow::Borrowed("llm_provider")) {
            // 添加自定义头，标识使用的提供商
            upstream_request.insert_header("X-LLM-Provider", HeaderValue::from_str(provider)?)?;
            
            // 可以在这里添加更多针对特定提供商的请求修改逻辑
            if provider.contains("anthropic") {
                upstream_request.insert_header("Anthropic-Version", HeaderValue::from_static("2023-06-01"))?;
            } else if provider.contains("openai") {
                // OpenAI特定头部
            }
        }
        
        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        _state: &mut RouterContext,
        _config: &RoutePlugin,
    ) -> Result<bool> {
        // 直接通过
        Ok(false)
    }

    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        state: &mut RouterContext,
    ) -> Result<()> {
        // 添加一个响应头，指示使用的LLM提供商
        if let Some(provider) = state.extensions.get(&Cow::Borrowed("llm_provider")) {
            upstream_response.insert_header("X-LLM-Provider-Used", HeaderValue::from_str(provider)?)?;
        }
        
        Ok(())
    }
}

// 在静态注册表中注册插件
pub static LLM_ROUTER: Lazy<LlmRouter> = Lazy::new(LlmRouter::new);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_llm_provider_from_str() {
        assert!(matches!(LlmProvider::from("openai"), LlmProvider::OpenAI));
        assert!(matches!(LlmProvider::from("anthropic"), LlmProvider::Anthropic));
        assert!(matches!(LlmProvider::from("vertex"), LlmProvider::GoogleVertexAI));
        assert!(matches!(LlmProvider::from("azure"), LlmProvider::AzureOpenAI));
        assert!(matches!(LlmProvider::from("unknown"), LlmProvider::Custom));
    }
} 