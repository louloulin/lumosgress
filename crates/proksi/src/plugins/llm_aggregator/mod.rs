use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use http::HeaderValue;
use pingora::{
    http::ResponseHeader,
    proxy::Session,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tracing::info;

// Import new Plugin trait and related types
use crate::plugins::core::{Plugin, PluginError, PluginStep};
use crate::proxy_server::HttpResponse;

use crate::proxy_server::https_proxy::RouterContext;

// 聚合策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    // 多模型并行调用，返回最快的结果
    FastestResult,
    // 多模型并行调用，根据权重或规则合并结果
    CombineResults,
    // 多模型串行调用，结果传递给下一个模型
    Chain,
}

impl Default for AggregationStrategy {
    fn default() -> Self {
        AggregationStrategy::FastestResult
    }
}

impl From<&str> for AggregationStrategy {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fastest" | "fastest_result" => AggregationStrategy::FastestResult,
            "combine" | "combine_results" => AggregationStrategy::CombineResults,
            "chain" | "chain_results" => AggregationStrategy::Chain,
            _ => AggregationStrategy::FastestResult, // 默认
        }
    }
}

// LLM聚合配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmAggregatorConfig {
    pub strategy: AggregationStrategy,
    pub providers: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub weights: Option<HashMap<String, f64>>,
}

// 聚合请求的状态
#[derive(Debug)]
struct AggregationState {
    results: HashMap<String, Value>,
    completed: usize,
    total: usize,
    start_time: std::time::Instant,
}

// Modify LlmAggregator struct
#[derive(Debug, Clone)]
pub struct LlmAggregator {
    pub config: LlmAggregatorConfig,
    // Use请求ID作为键存储聚合状态
    pub states: Arc<Mutex<HashMap<String, AggregationState>>>,
}

impl LlmAggregator {
    pub fn new() -> Self {
        Self {
            config: LlmAggregatorConfig::default(),
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn with_config(config: LlmAggregatorConfig) -> Self {
        Self { 
            config,
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // NOTE: This helper might be used if core supports aggregation
    async fn create_derived_requests(&self, session: &Session, providers: &[String]) -> Result<Vec<(String, Value)>> {
        let mut requests = Vec::new();
        
        // 读取原始请求的body
        if let Some(content_type) = session.req_header().headers.get("content-type") {
            if content_type.to_str().unwrap_or("").contains("application/json") {
                // 克隆原始请求体
                // 注意：实际实现时需要读取原始请求体，并根据不同提供商格式调整
                let dummy_body = json!({"message": "This is a placeholder for the real request body"});
                
                for provider in providers {
                    requests.push((provider.clone(), dummy_body.clone()));
                }
            }
        }
        
        Ok(requests)
    }
}

// Add new Plugin trait implementation structure
#[async_trait]
impl Plugin for LlmAggregator {
    fn name(&self) -> &'static str {
        "llm_aggregator"
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        _session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // Placeholder: Ideally runs in Request step to potentially initiate aggregation
        if step == PluginStep::Request {
            let strategy_str = format!("{:?}", self.config.strategy);
            info!(
                strategy = strategy_str,
                providers = ?self.config.providers,
                "LlmAggregator: Intending to use strategy (Actual aggregation NOT implemented due to core limitations)."
            );
            // Store intended strategy for response header
            ctx.plugins_data.insert(
                "llm_aggregation_strategy".to_string(), 
                Value::String(strategy_str)
            );
            // Store aggregation ID (could be used later if core supports it)
            let agg_id = uuid::Uuid::new_v4().to_string();
            ctx.plugins_data.insert(
                "llm_aggregator_id".to_string(),
                Value::String(agg_id)
            );
        }
        
        // Always continue the request for now, as we cannot block/spawn requests
        Ok((false, None)) 
    }

    async fn handle_response(
        &self,
        step: PluginStep,
        _session: &mut Session,
        ctx: &mut RouterContext,
        upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        // Add header in Response step based on stored strategy
        if step == PluginStep::Response {
            if let Some(strategy_val) = ctx.plugins_data.get("llm_aggregation_strategy") {
                if let Some(strategy) = strategy_val.as_str() {
                    upstream_response.insert_header("X-LLM-Aggregation", HeaderValue::from_str(strategy)?)?;
                    return Ok(true); // Header added
                }
            }
        }
        Ok(false)
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        Ok(()) // No specific start logic needed yet
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        Ok(()) // No specific stop logic needed yet
    }
}

/* // Commented out outdated implementation
#[async_trait]
impl MiddlewarePlugin for LlmAggregator {
    async fn request_filter(
        &self,
        _session: &mut Session,
        state: &mut RouterContext,
        config: &RoutePlugin,
    ) -> Result<bool> {
        // 获取插件配置
        let config_data = config.config.as_ref().ok_or_else(|| anyhow!("LLM Aggregator plugin requires configuration"))?;
        
        // 获取配置名
        let config_name = get_required_config(config_data, "config_name").unwrap_or_else(|_| "default".to_string());
        
        // 生成请求ID
        let request_id = uuid::Uuid::new_v4().to_string();
        state.extensions.insert(Cow::Borrowed("aggregator_request_id"), request_id);
        
        // 获取聚合策略并处理
        if let Some(aggregator_config) = self.config.get(&config_name) {
            // 记录策略
            info!(
                strategy = ?aggregator_config.strategy,
                "Using LLM aggregation strategy"
            );
            
            // 在当前Pingora API中，我们不能直接访问或修改response，
            // 因此这里只是模拟实现，后续需要适配Pingora API
            
            // 实际上这些策略的处理需要修改为适配Pingora的方式
            match aggregator_config.strategy {
                AggregationStrategy::FastestResult => {
                    info!("Using fastest result strategy (simulated)");
                    state.extensions.insert(Cow::Borrowed("llm_aggregation_strategy"), "fastest".to_string());
                },
                AggregationStrategy::CombineResults => {
                    info!("Using combine results strategy (simulated)");
                    state.extensions.insert(Cow::Borrowed("llm_aggregation_strategy"), "combine".to_string());
                },
                AggregationStrategy::Chain => {
                    info!("Using chain strategy (simulated)");
                    state.extensions.insert(Cow::Borrowed("llm_aggregation_strategy"), "chain".to_string());
                },
            }
        } else {
            warn!("LLM Aggregator config not found: {}", config_name);
        }
        
        // 继续处理请求
        Ok(false)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        _upstream_request: &mut RequestHeader,
        _state: &mut RouterContext,
    ) -> Result<()> {
        // 聚合已在request_filter中完成或记录
        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        _state: &mut RouterContext,
        _config: &RoutePlugin,
    ) -> Result<bool> {
        // 当前无需处理响应
        Ok(false)
    }

    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        state: &mut RouterContext,
    ) -> Result<()> {
        // 添加一个响应头，指示使用的聚合策略
        if let Some(strategy) = state.extensions.get(&Cow::Borrowed("llm_aggregation_strategy")) {
            upstream_response.insert_header("X-LLM-Aggregation", HeaderValue::from_str(strategy)?)?;
        }
        
        Ok(())
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::core::PluginStep;
    use crate::proxy_server::https_proxy::RouterContext;
    use pingora::proxy::Session;
    use http::HeaderValue;
    use std::collections::HashMap;
    use serde_json::Value;
    
    #[test]
    fn test_aggregation_strategy_from_str() {
        assert!(matches!(AggregationStrategy::from("fastest"), AggregationStrategy::FastestResult));
        assert!(matches!(AggregationStrategy::from("combine"), AggregationStrategy::CombineResults));
        assert!(matches!(AggregationStrategy::from("chain"), AggregationStrategy::Chain));
        assert!(matches!(AggregationStrategy::from("unknown"), AggregationStrategy::FastestResult));
    }
    
    fn create_test_context() -> RouterContext {
        RouterContext {
            host: "test.example.com".to_string(),
            route_container: Default::default(),
            upstream: Default::default(),
            extensions: HashMap::new(),
            is_websocket: false,
            timings: Default::default(),
            upstream_response: None,
            plugins_data: HashMap::new(),
            request_id: String::new(),
        }
    }
    
    #[tokio::test]
    async fn test_plugin_creation_with_config() {
        let config = LlmAggregatorConfig {
            strategy: AggregationStrategy::CombineResults,
            providers: vec!["openai".to_string(), "anthropic".to_string()],
            timeout_ms: Some(5000),
            weights: None,
        };
        
        let plugin = LlmAggregator::with_config(config.clone());
        assert!(matches!(plugin.config.strategy, AggregationStrategy::CombineResults));
        assert_eq!(plugin.config.providers.len(), 2);
        assert_eq!(plugin.states.lock().await.len(), 0);
    }
    
    /* // TODO: Fix mock session creation to re-enable this test
    #[tokio::test]
    async fn test_handle_response_adds_header() {
        let config = LlmAggregatorConfig {
            strategy: AggregationStrategy::FastestResult,
            providers: vec!["test".to_string()],
            timeout_ms: None,
            weights: None,
        };
        let plugin = LlmAggregator::with_config(config);
        let mut session = ???; // Need to figure out how to mock session
        let mut ctx = create_test_context();
        let mut upstream_response = ResponseHeader::build(200, Some(4)).unwrap();

        ctx.plugins_data.insert(
            "llm_aggregation_strategy".to_string(), 
            Value::String(format!("{:?}", plugin.config.strategy))
        );
        
        let result = plugin.handle_response(PluginStep::Response, &mut session, &mut ctx, &mut upstream_response).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        assert_eq!(
            upstream_response.headers.get("X-LLM-Aggregation").unwrap().to_str().unwrap(), 
            "FastestResult"
        );
    }
    */
    
    // NOTE: Testing the core aggregation logic in handle_request requires 
    // significant changes to the core plugin execution model.
} 