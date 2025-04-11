use std::{borrow::Cow, collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use http::{HeaderValue, StatusCode};
use once_cell::sync::Lazy;
use pingora::{
    http::{RequestHeader, ResponseHeader},
    proxy::Session,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::{
    config::RoutePlugin,
    plugins::get_required_config,
    proxy_server::https_proxy::RouterContext,
};

use super::MiddlewarePlugin;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAggregatorConfig {
    pub strategy: AggregationStrategy,
    pub providers: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub weights: Option<HashMap<String, f64>>,
}

// 聚合请求的状态
struct AggregationState {
    results: HashMap<String, Value>,
    completed: usize,
    total: usize,
    start_time: std::time::Instant,
}

pub struct LlmAggregator {
    pub config: Arc<HashMap<String, LlmAggregatorConfig>>,
    // 使用请求ID作为键存储聚合状态
    pub states: Arc<Mutex<HashMap<String, AggregationState>>>,
}

impl LlmAggregator {
    pub fn new() -> Self {
        Self {
            config: Arc::new(HashMap::new()),
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // 从请求创建派生请求
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

impl Clone for LlmAggregator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            states: self.states.clone(),
        }
    }
}

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

// 在静态注册表中注册插件
pub static LLM_AGGREGATOR: Lazy<LlmAggregator> = Lazy::new(LlmAggregator::new);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aggregation_strategy_from_str() {
        assert!(matches!(AggregationStrategy::from("fastest"), AggregationStrategy::FastestResult));
        assert!(matches!(AggregationStrategy::from("combine"), AggregationStrategy::CombineResults));
        assert!(matches!(AggregationStrategy::from("chain"), AggregationStrategy::Chain));
        assert!(matches!(AggregationStrategy::from("unknown"), AggregationStrategy::FastestResult));
    }
} 