use std::{collections::HashMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use anyhow::Result;
use async_trait::async_trait;
use bytes;
use chrono::{DateTime, Utc};
use http::StatusCode;
use pingora::{http::ResponseHeader, proxy::Session};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{plugins::core::{Plugin, PluginError, PluginStep, PluginMetadata, PluginType}, proxy_server::{https_proxy::RouterContext, HttpResponse}};

// Performance metrics for a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub request_size: usize,
    pub response_size: Option<usize>,
    pub total_duration_ms: u64,
    pub request_processing_ms: u64,
    pub upstream_latency_ms: u64,
    pub response_processing_ms: u64,
    pub token_count_input: Option<usize>,
    pub token_count_output: Option<usize>,
    pub error: Option<String>,
    pub path: String,
    pub method: String,
    pub status_code: Option<u16>,
    pub client_ip: Option<String>,
    pub tags: HashMap<String, String>,
}

// Performance metrics summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub avg_response_time_ms: f64,
    pub p50_response_time_ms: f64,
    pub p90_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub avg_upstream_latency_ms: f64,
    pub avg_request_size: f64,
    pub avg_response_size: f64,
    pub avg_tokens_input: f64,
    pub avg_tokens_output: f64,
    pub success_rate: f64,
    pub request_count: usize,
    pub error_count: usize,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

// Performance hot spots identified in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSpot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub impact_level: HotSpotImpact,
    pub avg_time_ms: f64,
    pub occurrence_count: usize,
    pub recommendation: String,
    pub identified_at: DateTime<Utc>,
}

// LLM性能优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOptimization {
    pub id: String,
    pub optimization_type: OptimizationType,
    pub description: String,
    pub potential_improvement: f64, // 预期改进百分比
    pub implementation_difficulty: DifficultyLevel,
    pub estimated_cost: CostLevel,
    pub recommendation: String,
    pub metrics_before: Option<MetricsSummary>,
    pub metrics_after: Option<MetricsSummary>,
    pub created_at: DateTime<Utc>,
    pub applied_at: Option<DateTime<Utc>>,
    pub status: OptimizationStatus,
}

// 优化类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    Caching,           // 缓存优化
    RequestBatching,   // 请求批处理
    ModelSelection,    // 模型选择优化
    TokenOptimization, // Token优化
    LoadBalancing,     // 负载均衡
    ConnectionPooling, // 连接池优化
    ResponseStreaming, // 响应流优化
    PromptOptimization, // 提示词优化
    Custom(String),    // 自定义优化
}

// 实施难度级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Easy,    // 容易实施
    Medium,  // 中等难度
    Hard,    // 困难
    Expert,  // 需要专家级别
}

// 成本级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostLevel {
    Free,     // 免费
    Low,      // 低成本
    Medium,   // 中等成本
    High,     // 高成本
    VeryHigh, // 非常高成本
}

// 优化状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStatus {
    Suggested,    // 建议中
    InProgress,   // 实施中
    Applied,      // 已应用
    Reverted,     // 已回滚
    Failed,       // 实施失败
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotSpotImpact {
    Low,
    Medium,
    High,
    Critical,
}

// Storage backend options for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricsStorage {
    Memory { max_entries: usize },
    File { path: String },
    Redis { url: String, key_prefix: String },
    Prometheus { endpoint: String },
}

// Configuration for the Performance Analyzer plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalyzerConfig {
    pub enabled: bool,
    pub ui_endpoint: String,
    pub sample_rate: f64,
    pub detailed_profiling: bool,
    pub storage: MetricsStorage,
    pub hotspot_detection: bool,
    pub profile_token_usage: bool,
    pub max_trace_depth: Option<usize>,
    pub exclude_paths: Vec<String>,
    pub trace_headers: bool,
    // 新增：性能优化配置
    pub auto_optimization: bool,
    pub optimization_thresholds: OptimizationThresholds,
    pub caching_config: CachingConfig,
    pub batching_config: BatchingConfig,
    pub model_selection_config: ModelSelectionConfig,
}

// 优化阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationThresholds {
    pub response_time_ms: f64,      // 响应时间阈值
    pub error_rate_percent: f64,    // 错误率阈值
    pub token_efficiency: f64,      // Token效率阈值
    pub cost_per_request: f64,      // 每请求成本阈值
    pub throughput_rps: f64,        // 吞吐量阈值
}

// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_cache_size: usize,
    pub cache_key_strategy: CacheKeyStrategy,
    pub cache_hit_threshold: f64,
}

// 缓存键策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheKeyStrategy {
    FullRequest,      // 完整请求作为键
    PromptOnly,       // 仅提示词作为键
    SemanticHash,     // 语义哈希
    Custom(String),   // 自定义策略
}

// 批处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchingConfig {
    pub enabled: bool,
    pub max_batch_size: usize,
    pub batch_timeout_ms: u64,
    pub compatible_models: Vec<String>,
}

// 模型选择配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectionConfig {
    pub enabled: bool,
    pub auto_fallback: bool,
    pub performance_weights: ModelPerformanceWeights,
    pub model_rankings: HashMap<String, ModelRanking>,
}

// 模型性能权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceWeights {
    pub speed: f64,      // 速度权重
    pub accuracy: f64,   // 准确性权重
    pub cost: f64,       // 成本权重
    pub reliability: f64, // 可靠性权重
}

// 模型排名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRanking {
    pub model_name: String,
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
    pub cost_per_token: f64,
    pub quality_score: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for PerformanceAnalyzerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ui_endpoint: "/performance-analyzer".to_string(),
            sample_rate: 1.0, // Sample all requests
            detailed_profiling: false,
            storage: MetricsStorage::Memory { max_entries: 10000 },
            hotspot_detection: true,
            profile_token_usage: true,
            max_trace_depth: Some(10),
            exclude_paths: vec!["/health".to_string(), "/metrics".to_string()],
            trace_headers: false,
            auto_optimization: true,
            optimization_thresholds: OptimizationThresholds::default(),
            caching_config: CachingConfig::default(),
            batching_config: BatchingConfig::default(),
            model_selection_config: ModelSelectionConfig::default(),
        }
    }
}

impl Default for OptimizationThresholds {
    fn default() -> Self {
        Self {
            response_time_ms: 2000.0,      // 2秒响应时间阈值
            error_rate_percent: 5.0,       // 5%错误率阈值
            token_efficiency: 0.8,         // 80%Token效率阈值
            cost_per_request: 0.01,        // $0.01每请求成本阈值
            throughput_rps: 100.0,         // 100 RPS吞吐量阈值
        }
    }
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 3600,             // 1小时TTL
            max_cache_size: 10000,         // 最大缓存10000条
            cache_key_strategy: CacheKeyStrategy::SemanticHash,
            cache_hit_threshold: 0.7,      // 70%缓存命中率阈值
        }
    }
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            enabled: false,                // 默认关闭批处理
            max_batch_size: 10,
            batch_timeout_ms: 100,         // 100ms批处理超时
            compatible_models: vec![
                "gpt-3.5-turbo".to_string(),
                "gpt-4".to_string(),
            ],
        }
    }
}

impl Default for ModelSelectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_fallback: true,
            performance_weights: ModelPerformanceWeights::default(),
            model_rankings: HashMap::new(),
        }
    }
}

impl Default for ModelPerformanceWeights {
    fn default() -> Self {
        Self {
            speed: 0.3,        // 30%权重给速度
            accuracy: 0.4,     // 40%权重给准确性
            cost: 0.2,         // 20%权重给成本
            reliability: 0.1,  // 10%权重给可靠性
        }
    }
}

// Context data stored in plugins_data
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PerformancePluginContext {
    is_sampled: bool,
    start_time_unix_ns: u128,
    request_processing_end_unix_ns: Option<u128>,
    upstream_request_start_unix_ns: Option<u128>,
    upstream_response_received_unix_ns: Option<u128>,
}

impl PerformancePluginContext {
    fn new(is_sampled: bool) -> Self {
        Self {
            is_sampled,
            start_time_unix_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos(),
            request_processing_end_unix_ns: None,
            upstream_request_start_unix_ns: None,
            upstream_response_received_unix_ns: None,
        }
    }

    // Helper to get current time as u128 nanos since epoch
    fn now_ns() -> u128 {
         SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()
    }

    // Helper to calculate duration in ms from Option<u128> ns timestamps
    fn duration_ms(start_ns: Option<u128>, end_ns: Option<u128>) -> u64 {
        match (start_ns, end_ns) {
            (Some(start), Some(end)) if end >= start => ((end - start) / 1_000_000) as u64,
            _ => 0, // Return 0 if timestamps are missing or invalid
        }
    }
}

// Main Performance Analyzer plugin
#[derive(Debug, Default)]
pub struct PerformanceAnalyzer {
    config: Arc<Mutex<PerformanceAnalyzerConfig>>,
    metrics: Arc<Mutex<Vec<RequestMetrics>>>,
    hot_spots: Arc<Mutex<Vec<HotSpot>>>,
    // 新增：性能优化相关字段
    optimizations: Arc<Mutex<Vec<PerformanceOptimization>>>,
    cache: Arc<Mutex<HashMap<String, (String, DateTime<Utc>)>>>, // 简单的内存缓存
    model_performance: Arc<Mutex<HashMap<String, ModelRanking>>>, // 模型性能统计
    batch_queue: Arc<Mutex<Vec<BatchRequest>>>, // 批处理队列
}

// 批处理请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    pub id: String,
    pub request_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub callback: Option<String>, // 回调URL或标识
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(PerformanceAnalyzerConfig::default())),
            metrics: Arc::new(Mutex::new(Vec::new())),
            hot_spots: Arc::new(Mutex::new(Vec::new())),
            optimizations: Arc::new(Mutex::new(Vec::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
            model_performance: Arc::new(Mutex::new(HashMap::new())),
            batch_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_config(config: PerformanceAnalyzerConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            metrics: Arc::new(Mutex::new(Vec::new())),
            hot_spots: Arc::new(Mutex::new(Vec::new())),
            optimizations: Arc::new(Mutex::new(Vec::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
            model_performance: Arc::new(Mutex::new(HashMap::new())),
            batch_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Finalize and store metrics for a completed request
    async fn finalize_metrics(&self, session: &Session, ctx: &mut RouterContext, config: &PerformanceAnalyzerConfig) -> Result<()> {
         const CONTEXT_KEY: &str = "performance_analyzer_context";
         let perf_ctx_val = ctx.plugins_data.get(CONTEXT_KEY);

         if perf_ctx_val.is_none() {
             debug!("No performance context found for request ID: {}, skipping metrics finalization.", ctx.request_id);
             return Ok(()); // Not sampled or context missing
         }

         let perf_ctx: PerformancePluginContext = serde_json::from_value(perf_ctx_val.unwrap().clone())?;

         if !perf_ctx.is_sampled {
              debug!("Request ID {} was not sampled, skipping metrics finalization.", ctx.request_id);
            return Ok(());
        }
        
         let final_time_ns = PerformancePluginContext::now_ns();

         // Calculate durations
         let total_duration_ms = PerformancePluginContext::duration_ms(Some(perf_ctx.start_time_unix_ns), Some(final_time_ns));
         let request_processing_ms = PerformancePluginContext::duration_ms(Some(perf_ctx.start_time_unix_ns), perf_ctx.request_processing_end_unix_ns);
         let upstream_latency_ms = PerformancePluginContext::duration_ms(perf_ctx.upstream_request_start_unix_ns, perf_ctx.upstream_response_received_unix_ns);
         // Response processing is the time from upstream response received to now (end of Log step)
         let response_processing_ms = PerformancePluginContext::duration_ms(perf_ctx.upstream_response_received_unix_ns, Some(final_time_ns));

         let req_header = session.req_header();
         let resp_header = ctx.upstream_response.as_ref(); // Use response from context

         // Extract data
         let request_size = req_header.headers.get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<usize>().ok())
             .unwrap_or(0);

         let response_size = resp_header.and_then(|h| h.headers.get("content-length")
                .and_then(|v| v.to_str().ok())
             .and_then(|s| s.parse::<usize>().ok()));

         let status_code = resp_header.map(|h| h.status.as_u16());
         let path = req_header.uri.path().to_string();
         let method = req_header.method.as_str().to_string();
         let client_ip = session.client_addr().map(|sa| {
             match sa {
                 pingora::protocols::l4::socket::SocketAddr::Inet(addr) => addr.ip().to_string(),
                 pingora::protocols::l4::socket::SocketAddr::Unix(_) => "unix_socket".to_string(),
             }
         });

         // Extract provider/model from context if available
         let provider = ctx.plugins_data.get("llm_provider").and_then(|v| v.as_str().map(String::from));
         let model = ctx.plugins_data.get("llm_model").and_then(|v| v.as_str().map(String::from));

         // Placeholder for token counts (requires body access)
         let token_count_input = if config.profile_token_usage { None } else { None };
         let token_count_output = if config.profile_token_usage { None } else { None };

         let error = if status_code.map_or(false, |s| s >= 400) {
             // Basic error detection from status code
             Some(format!("HTTP Status {}", status_code.unwrap()))
         } else {
             None
         };

         let metrics = RequestMetrics {
             request_id: ctx.request_id.clone(),
             timestamp: Utc::now(),
             provider,
             model,
             request_size,
             response_size,
             total_duration_ms,
             request_processing_ms,
             upstream_latency_ms,
             response_processing_ms,
             token_count_input,
             token_count_output,
             error,
             path,
             method,
             status_code,
             client_ip,
             tags: HashMap::new(), // Add relevant tags if needed
        };
        
        // Store metrics
         self.store_metrics(metrics, config).await?; // Call the storage logic

         // Optionally run hotspot detection (could be run periodically instead)
         if config.hotspot_detection {
             self.detect_hotspots().await?; // Run detection based on stored metrics
         }
        
        Ok(())
    }

    // Store collected metrics based on configuration
    async fn store_metrics(&self, metrics: RequestMetrics, config: &PerformanceAnalyzerConfig) -> Result<()> {
        match &config.storage {
            MetricsStorage::Memory { max_entries } => {
                let mut stored_metrics = self.metrics.lock().await;
                stored_metrics.push(metrics);
                // Prune if exceeding max entries
                if stored_metrics.len() > *max_entries {
                    let overflow = stored_metrics.len() - *max_entries;
                    stored_metrics.drain(0..overflow); // Remove oldest
                }
            }
            MetricsStorage::File { path: _ } => {
                // TODO: Implement file storage (append to file, handle rotation)
                warn!("File storage for performance metrics not yet implemented.");
            }
            MetricsStorage::Redis { url: _, key_prefix: _ } => {
                // TODO: Implement Redis storage (connect, store metric)
                 warn!("Redis storage for performance metrics not yet implemented.");
            }
            MetricsStorage::Prometheus { endpoint: _ } => {
                // TODO: Implement Prometheus exporter (update gauges/counters)
                 warn!("Prometheus storage for performance metrics not yet implemented.");
            }
        }
        Ok(())
    }
    
    // Detect performance hotspots based on stored metrics (simplified)
    async fn detect_hotspots(&self) -> Result<()> {
         // Basic hotspot detection logic (example: find slowest paths)
         let metrics_snapshot = self.metrics.lock().await.clone(); // Clone for analysis
         if metrics_snapshot.len() < 50 { // Need enough data
             return Ok(());
         }

         let mut path_timings: HashMap<String, Vec<u64>> = HashMap::new();
         for metric in metrics_snapshot {
             path_timings.entry(metric.path.clone()).or_default().push(metric.total_duration_ms);
         }

         let mut potential_hotspots = vec![];
         for (path, timings) in path_timings {
            if timings.len() > 10 { // Consider paths with enough requests
                 let avg_time = timings.iter().sum::<u64>() as f64 / timings.len() as f64;
                 if avg_time > 1000.0 { // Example threshold: > 1 second average
                     potential_hotspots.push(HotSpot {
                         id: Uuid::new_v4().to_string(),
                         name: format!("Slow Path: {}", path),
                         description: format!("Requests to path '{}' have an average response time of {:.2} ms.", path, avg_time),
                         impact_level: if avg_time > 5000.0 { HotSpotImpact::High } else { HotSpotImpact::Medium },
                         avg_time_ms: avg_time,
                         occurrence_count: timings.len(),
                         recommendation: "Investigate the backend service or plugins affecting this path.".to_string(),
                         identified_at: Utc::now(),
                     });
                 }
             }
         }

         if !potential_hotspots.is_empty() {
             let mut current_hotspots = self.hot_spots.lock().await;
             // Simple update: replace old hotspots with newly detected ones
             *current_hotspots = potential_hotspots;
             info!("Detected {} potential performance hotspots.", current_hotspots.len());
         }

        Ok(())
    }

    // 缓存相关方法
    async fn check_cache(&self, cache_key: &str) -> Option<String> {
        let cache = self.cache.lock().await;
        if let Some((cached_response, cached_at)) = cache.get(cache_key) {
            let config = self.config.lock().await;
            let ttl_duration = chrono::Duration::seconds(config.caching_config.ttl_seconds as i64);

            if Utc::now() - *cached_at < ttl_duration {
                info!("Cache hit for key: {}", cache_key);
                return Some(cached_response.clone());
            } else {
                info!("Cache expired for key: {}", cache_key);
            }
        }
        None
    }

    async fn store_in_cache(&self, cache_key: String, response: String) {
        let mut cache = self.cache.lock().await;
        let config = self.config.lock().await;

        // 检查缓存大小限制
        if cache.len() >= config.caching_config.max_cache_size {
            // 简单的LRU：移除最旧的条目
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(cache_key.clone(), (response, Utc::now()));
        info!("Stored response in cache for key: {}", cache_key);
    }

    fn generate_cache_key(&self, request_data: &serde_json::Value, strategy: &CacheKeyStrategy) -> String {
        match strategy {
            CacheKeyStrategy::FullRequest => {
                // 使用完整请求的哈希作为键
                format!("full_{}", self.hash_json(request_data))
            },
            CacheKeyStrategy::PromptOnly => {
                // 仅使用提示词部分
                if let Some(messages) = request_data.get("messages").and_then(|m| m.as_array()) {
                    let prompt_content = messages.iter()
                        .filter_map(|msg| msg.get("content").and_then(|c| c.as_str()))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("prompt_{}", self.hash_string(&prompt_content))
                } else if let Some(prompt) = request_data.get("prompt").and_then(|p| p.as_str()) {
                    format!("prompt_{}", self.hash_string(prompt))
                } else {
                    format!("unknown_{}", self.hash_json(request_data))
                }
            },
            CacheKeyStrategy::SemanticHash => {
                // 语义哈希（简化版本）
                format!("semantic_{}", self.semantic_hash(request_data))
            },
            CacheKeyStrategy::Custom(pattern) => {
                // 自定义策略
                format!("custom_{}_{}", pattern, self.hash_json(request_data))
            },
        }
    }

    fn hash_json(&self, data: &serde_json::Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let json_str = serde_json::to_string(data).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        json_str.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn hash_string(&self, data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn semantic_hash(&self, data: &serde_json::Value) -> String {
        // 简化的语义哈希：提取关键词并排序
        let text = self.extract_text_content(data);
        let words: std::collections::BTreeSet<&str> = text
            .split_whitespace()
            .filter(|word| word.len() > 3) // 过滤短词
            .collect();

        let semantic_key = words.into_iter().collect::<Vec<_>>().join("_");
        self.hash_string(&semantic_key)
    }

    fn extract_text_content(&self, data: &serde_json::Value) -> String {
        match data {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => {
                arr.iter().map(|v| self.extract_text_content(v)).collect::<Vec<_>>().join(" ")
            },
            serde_json::Value::Object(obj) => {
                obj.values().map(|v| self.extract_text_content(v)).collect::<Vec<_>>().join(" ")
            },
            _ => String::new(),
        }
    }

    // Serve the basic UI for the performance analyzer
    async fn serve_ui(&self, session: &mut Session, _ctx: &mut RouterContext, config: &PerformanceAnalyzerConfig) -> Result<ResponseHeader> {
        info!("Serving PerformanceAnalyzer UI");
         // Generate summary and hotspots based on current data
         let metrics_snapshot = self.metrics.lock().await.clone();
         let summary = self.generate_summary(&metrics_snapshot);
         let hotspots_snapshot = self.hot_spots.lock().await.clone();

         let summary_html = format!(
             "<h3>Metrics Summary (Last {} requests)</h3><pre>{}</pre>",
             metrics_snapshot.len(),
             serde_json::to_string_pretty(&summary).unwrap_or_else(|e| format!("Error generating summary: {}", e))
         );

         let hotspots_html = if config.hotspot_detection && !hotspots_snapshot.is_empty() {
             format!(
                 "<h3>Detected Hotspots</h3><pre>{}</pre>",
                 serde_json::to_string_pretty(&hotspots_snapshot).unwrap_or_else(|e| format!("Error generating hotspots: {}", e))
             )
         } else if config.hotspot_detection {
             "<h3>Detected Hotspots</h3><p>No significant hotspots detected yet.</p>".to_string()
         } else {
              "<h3>Hotspot Detection Disabled</h3>".to_string()
         };

         // Get storage type description
         let storage_desc = match config.storage {
             MetricsStorage::Memory { max_entries } => format!("Memory (max {} entries)", max_entries),
             MetricsStorage::File { ref path } => format!("File ({})", path),
             MetricsStorage::Redis { ref url, .. } => format!("Redis ({})", url),
             MetricsStorage::Prometheus { ref endpoint } => format!("Prometheus ({})", endpoint),
         };

         let html = format!(r#"
 <!DOCTYPE html>
 <html lang="en">
            <head>
     <meta charset="UTF-8">
     <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Performance Analyzer</title>
                <style>
         body {{ font-family: sans-serif; padding: 20px; }}
         pre {{ background-color: #f4f4f4; padding: 15px; border-radius: 5px; overflow-x: auto; }}
         h3 {{ border-bottom: 1px solid #eee; padding-bottom: 5px; margin-top: 30px; }}
                </style>
            </head>
            <body>
     <h1>Performance Analyzer</h1>
     <p>Displaying metrics based on configured storage (currently {}). Sampling rate: {}.</p>
     {}
     {}
     <p><a href="{}/api/metrics">View Raw Metrics (JSON)</a></p>
     <p><a href="{}/api/summary">View Summary (JSON)</a></p>
     <p><a href="{}/api/hotspots">View Hotspots (JSON)</a></p>
            </body>
 </html>
         "#,
         storage_desc,
         config.sample_rate,
         summary_html,
         hotspots_html,
         config.ui_endpoint, config.ui_endpoint, config.ui_endpoint // Links for API endpoints
         );

        let content_length = html.len();
        let mut response = ResponseHeader::build(StatusCode::OK, None)?;
        response.append_header("Content-Type", "text/html; charset=utf-8")?;
        response.append_header("Content-Length", content_length.to_string())?;

        session.write_response_header(Box::new(response.clone()), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(html)), true).await?;

        Ok(response)
    }

    // Handle API requests
    async fn handle_api_request(&self, session: &mut Session, _ctx: &mut RouterContext, path_suffix: &str, config: &PerformanceAnalyzerConfig) -> Result<ResponseHeader> {
        let (data, status) = match path_suffix {
            "/api/metrics" => {
                let metrics_snapshot = self.metrics.lock().await.clone();
                (serde_json::to_value(metrics_snapshot)?, StatusCode::OK)
            }
            "/api/summary" => {
                 let metrics_snapshot = self.metrics.lock().await.clone();
                 let summary = self.generate_summary(&metrics_snapshot);
                 (serde_json::to_value(summary)?, StatusCode::OK)
            }
            "/api/hotspots" => {
                 if config.hotspot_detection {
                    let hotspots_snapshot = self.hot_spots.lock().await.clone();
                    (serde_json::to_value(hotspots_snapshot)?, StatusCode::OK)
                 } else {
                     (json!({ "error": "Hotspot detection is disabled" }), StatusCode::SERVICE_UNAVAILABLE)
                 }
            }
            "/api/optimizations" => {
                let optimizations_snapshot = self.optimizations.lock().await.clone();
                (serde_json::to_value(optimizations_snapshot)?, StatusCode::OK)
            }
            "/api/model-performance" => {
                let model_perf_snapshot = self.model_performance.lock().await.clone();
                (serde_json::to_value(model_perf_snapshot)?, StatusCode::OK)
            }
            "/api/cache-stats" => {
                let cache_stats = self.get_cache_stats().await;
                (serde_json::to_value(cache_stats)?, StatusCode::OK)
            }
            "/api/generate-optimizations" => {
                let new_optimizations = self.generate_optimization_suggestions().await?;
                (serde_json::to_value(new_optimizations)?, StatusCode::OK)
            }
            _ => {
                 warn!("Unhandled PerformanceAnalyzer API path: {}", path_suffix);
                 (json!({ "error": "API endpoint not found" }), StatusCode::NOT_FOUND)
            }
        };

        let body_bytes = bytes::Bytes::from(serde_json::to_vec(&data)?);
        let mut response_header = ResponseHeader::build(status, None)?;
        response_header.append_header("Content-Type", "application/json")?;
        response_header.append_header("Content-Length", body_bytes.len().to_string())?;

        session.write_response_header(Box::new(response_header.clone()), false).await?;
        session.write_response_body(Some(body_bytes), true).await?;

        Ok(response_header)
    }

    // 获取缓存统计信息
    async fn get_cache_stats(&self) -> serde_json::Value {
        let cache = self.cache.lock().await;
        let config = self.config.lock().await;

        let total_entries = cache.len();
        let max_entries = config.caching_config.max_cache_size;
        let cache_usage_percent = if max_entries > 0 {
            (total_entries as f64 / max_entries as f64) * 100.0
        } else {
            0.0
        };

        json!({
            "total_entries": total_entries,
            "max_entries": max_entries,
            "cache_usage_percent": cache_usage_percent,
            "ttl_seconds": config.caching_config.ttl_seconds,
            "cache_key_strategy": config.caching_config.cache_key_strategy,
            "enabled": config.caching_config.enabled
        })
    }

    // 生成性能优化建议
    async fn generate_optimization_suggestions(&self) -> Result<Vec<PerformanceOptimization>> {
        let metrics = self.metrics.lock().await.clone();
        let config = self.config.lock().await;
        let mut optimizations = Vec::new();

        if metrics.is_empty() {
            return Ok(optimizations);
        }

        let summary = self.generate_summary(&metrics);

        // 响应时间优化建议
        if summary.avg_response_time_ms > config.optimization_thresholds.response_time_ms {
            optimizations.push(PerformanceOptimization {
                id: Uuid::new_v4().to_string(),
                optimization_type: OptimizationType::Caching,
                description: format!(
                    "平均响应时间 {:.2}ms 超过阈值 {:.2}ms，建议启用缓存",
                    summary.avg_response_time_ms,
                    config.optimization_thresholds.response_time_ms
                ),
                potential_improvement: 30.0, // 预期30%改进
                implementation_difficulty: DifficultyLevel::Easy,
                estimated_cost: CostLevel::Low,
                recommendation: "启用响应缓存，特别是对于重复的查询请求".to_string(),
                metrics_before: Some(summary.clone()),
                metrics_after: None,
                created_at: Utc::now(),
                applied_at: None,
                status: OptimizationStatus::Suggested,
            });
        }

        // Token效率优化建议
        if summary.avg_tokens_input > 0.0 {
            let token_efficiency = summary.avg_tokens_output / summary.avg_tokens_input;
            if token_efficiency < config.optimization_thresholds.token_efficiency {
                optimizations.push(PerformanceOptimization {
                    id: Uuid::new_v4().to_string(),
                    optimization_type: OptimizationType::TokenOptimization,
                    description: format!(
                        "Token效率 {:.2} 低于阈值 {:.2}，建议优化提示词",
                        token_efficiency,
                        config.optimization_thresholds.token_efficiency
                    ),
                    potential_improvement: 25.0,
                    implementation_difficulty: DifficultyLevel::Medium,
                    estimated_cost: CostLevel::Free,
                    recommendation: "优化提示词长度，使用更精确的指令，减少不必要的上下文".to_string(),
                    metrics_before: Some(summary.clone()),
                    metrics_after: None,
                    created_at: Utc::now(),
                    applied_at: None,
                    status: OptimizationStatus::Suggested,
                });
            }
        }

        // 错误率优化建议
        if summary.success_rate < (1.0 - config.optimization_thresholds.error_rate_percent / 100.0) {
            optimizations.push(PerformanceOptimization {
                id: Uuid::new_v4().to_string(),
                optimization_type: OptimizationType::LoadBalancing,
                description: format!(
                    "成功率 {:.2}% 低于预期，建议改进负载均衡和错误处理",
                    summary.success_rate * 100.0
                ),
                potential_improvement: 15.0,
                implementation_difficulty: DifficultyLevel::Hard,
                estimated_cost: CostLevel::Medium,
                recommendation: "实施智能负载均衡，添加重试机制，改进错误处理".to_string(),
                metrics_before: Some(summary.clone()),
                metrics_after: None,
                created_at: Utc::now(),
                applied_at: None,
                status: OptimizationStatus::Suggested,
            });
        }

        // 模型选择优化建议
        let model_performance = self.analyze_model_performance(&metrics).await;
        if model_performance.len() > 1 {
            // 如果有多个模型，建议使用性能最好的
            if let Some(best_model) = model_performance.values().min_by(|a, b| {
                a.avg_response_time_ms.partial_cmp(&b.avg_response_time_ms).unwrap_or(std::cmp::Ordering::Equal)
            }) {
                optimizations.push(PerformanceOptimization {
                    id: Uuid::new_v4().to_string(),
                    optimization_type: OptimizationType::ModelSelection,
                    description: format!(
                        "建议优先使用性能最佳的模型: {}",
                        best_model.model_name
                    ),
                    potential_improvement: 20.0,
                    implementation_difficulty: DifficultyLevel::Easy,
                    estimated_cost: CostLevel::Free,
                    recommendation: format!(
                        "将 {} 设置为默认模型，平均响应时间 {:.2}ms，成功率 {:.2}%",
                        best_model.model_name,
                        best_model.avg_response_time_ms,
                        best_model.success_rate * 100.0
                    ),
                    metrics_before: Some(summary.clone()),
                    metrics_after: None,
                    created_at: Utc::now(),
                    applied_at: None,
                    status: OptimizationStatus::Suggested,
                });
            }
        }

        // 存储优化建议
        let mut stored_optimizations = self.optimizations.lock().await;
        stored_optimizations.extend(optimizations.clone());

        Ok(optimizations)
    }

    // 分析模型性能
    async fn analyze_model_performance(&self, metrics: &[RequestMetrics]) -> HashMap<String, ModelRanking> {
        let mut model_stats: HashMap<String, Vec<&RequestMetrics>> = HashMap::new();

        // 按模型分组指标
        for metric in metrics {
            if let Some(model) = &metric.model {
                model_stats.entry(model.clone()).or_default().push(metric);
            }
        }

        let mut model_rankings = HashMap::new();

        for (model_name, model_metrics) in model_stats {
            if model_metrics.is_empty() {
                continue;
            }

            let total_requests = model_metrics.len();
            let successful_requests = model_metrics.iter()
                .filter(|m| m.error.is_none() && m.status_code.map_or(true, |s| s < 400))
                .count();

            let avg_response_time = model_metrics.iter()
                .map(|m| m.total_duration_ms)
                .sum::<u64>() as f64 / total_requests as f64;

            let success_rate = successful_requests as f64 / total_requests as f64;

            // 简化的质量评分（基于响应时间和成功率）
            let quality_score = (success_rate * 0.7) + ((1.0 - (avg_response_time / 10000.0).min(1.0)) * 0.3);

            let ranking = ModelRanking {
                model_name: model_name.clone(),
                avg_response_time_ms: avg_response_time,
                success_rate,
                cost_per_token: 0.0, // 需要外部数据源
                quality_score,
                last_updated: Utc::now(),
            };

            model_rankings.insert(model_name, ranking);
        }

        // 更新存储的模型性能数据
        let mut stored_performance = self.model_performance.lock().await;
        stored_performance.extend(model_rankings.clone());

        model_rankings
    }

    // Generate summary statistics from collected metrics
    fn generate_summary(&self, metrics: &[RequestMetrics]) -> MetricsSummary {
        if metrics.is_empty() {
            return MetricsSummary {
                avg_response_time_ms: 0.0, p50_response_time_ms: 0.0, p90_response_time_ms: 0.0,
                p95_response_time_ms: 0.0, p99_response_time_ms: 0.0, avg_upstream_latency_ms: 0.0,
                avg_request_size: 0.0, avg_response_size: 0.0, avg_tokens_input: 0.0,
                avg_tokens_output: 0.0, success_rate: 1.0, request_count: 0, error_count: 0,
                period_start: Utc::now(), period_end: Utc::now(),
            };
        }

        let mut response_times: Vec<u64> = metrics.iter().map(|m| m.total_duration_ms).collect();
        response_times.sort_unstable();

        let request_count = metrics.len();
        let error_count = metrics.iter().filter(|m| m.error.is_some() || m.status_code.map_or(false, |s| s >= 400)).count();
        let success_rate = if request_count > 0 { 1.0 - (error_count as f64 / request_count as f64) } else { 1.0 };

        let avg_response_time_ms = response_times.iter().sum::<u64>() as f64 / request_count as f64;
        let avg_upstream_latency_ms = metrics.iter().map(|m| m.upstream_latency_ms).sum::<u64>() as f64 / request_count as f64;
        let avg_request_size = metrics.iter().map(|m| m.request_size).sum::<usize>() as f64 / request_count as f64;
        let avg_response_size = metrics.iter().filter_map(|m| m.response_size).sum::<usize>() as f64 / request_count as f64;
        let avg_tokens_input = metrics.iter().filter_map(|m| m.token_count_input).sum::<usize>() as f64 / request_count as f64;
        let avg_tokens_output = metrics.iter().filter_map(|m| m.token_count_output).sum::<usize>() as f64 / request_count as f64;

        let percentile = |p: f64| {
            if response_times.is_empty() { return 0.0; }
            let index = (p * (response_times.len() - 1) as f64).round() as usize;
            response_times.get(index).copied().unwrap_or(0) as f64
        };

        MetricsSummary {
            avg_response_time_ms,
            p50_response_time_ms: percentile(0.50),
            p90_response_time_ms: percentile(0.90),
            p95_response_time_ms: percentile(0.95),
            p99_response_time_ms: percentile(0.99),
            avg_upstream_latency_ms,
            avg_request_size,
            avg_response_size,
            avg_tokens_input,
            avg_tokens_output,
            success_rate,
            request_count,
            error_count,
            period_start: metrics.first().map_or_else(Utc::now, |m| m.timestamp),
            period_end: metrics.last().map_or_else(Utc::now, |m| m.timestamp),
        }
    }
}

#[async_trait]
impl Plugin for PerformanceAnalyzer {
    fn name(&self) -> &'static str {
        "performance_analyzer"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 450, // Performance analysis should run after most other plugins
            plugin_type: PluginType::Native,
            description: "Performance monitoring and analysis with metrics collection, caching, and hotspot detection".to_string(),
            author: "Proksi Team".to_string(),
            homepage: Some("https://github.com/luizfonseca/proksi".to_string()),
        }
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting PerformanceAnalyzer plugin");
        let config = self.config.lock().await;
        if config.enabled {
            info!("PerformanceAnalyzer plugin started with metrics enabled: {}", config.metrics_enabled);
            info!("Caching enabled: {}", config.caching_enabled);
            info!("Hotspot detection enabled: {}", config.hotspot_detection);
            info!("Max metrics stored: {}", config.max_metrics);
        } else {
            info!("PerformanceAnalyzer plugin is disabled");
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping PerformanceAnalyzer plugin");
        Ok(())
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        let config = self.config.lock().await.clone();
        
        if !config.enabled {
            return Ok((false, None));
        }
        
        let req = session.req_header();
        let req_path = req.uri.path().to_string();
        
        // Check if path is excluded
        if config.exclude_paths.iter().any(|path| req_path.starts_with(path)) {
            return Ok((false, None));
        }
        
        // Handle UI endpoints
        if req_path.starts_with(&config.ui_endpoint) {
            let path_suffix = req_path.strip_prefix(&config.ui_endpoint)
                .unwrap_or("")
                .trim_start_matches('/');
                
            if path_suffix.is_empty() || path_suffix == "index.html" {
                match self.serve_ui(session, ctx, &config).await {
                    Ok(response) => {
                        let http_response = HttpResponse {
                            status_code: response.status,
                            headers: response.headers.clone(),
                            body: bytes::Bytes::new(), // Body was already written in serve_ui
                        };
                        return Ok((true, Some(http_response)));
                    },
                    Err(e) => {
                        error!("Failed to serve UI: {}", e);
                        return Ok((false, None));
                    }
                }
            }
            
            // Handle API requests
            if path_suffix.starts_with("api/") {
                match self.handle_api_request(session, ctx, path_suffix, &config).await {
                    Ok(response) => {
                        let http_response = HttpResponse {
                            status_code: response.status,
                            headers: response.headers.clone(),
                            body: bytes::Bytes::new(), // Body was already written in handle_api_request
                        };
                        return Ok((true, Some(http_response)));
                    },
                    Err(e) => {
                        error!("Failed to handle API request: {}", e);
                        return Ok((false, None));
                    }
                }
            }
        }
        
        // Process request based on step
        match step {
            PluginStep::Request => {
                // Sample rate check - only process some percentage of requests
                let is_sampled = rand::thread_rng().gen_bool(config.sample_rate);
                if !is_sampled {
                    return Ok((false, None));
                }
                
                // Initialize context data
                let perf_ctx = PerformancePluginContext::new(is_sampled);
                ctx.plugins_data.insert(
                    "performance_analyzer_context".to_string(),
                    serde_json::to_value(perf_ctx)?
                );
                
                // Store request info
                let req_header = session.req_header();
                ctx.plugins_data.insert(
                    "performance_analyzer_path".to_string(),
                    serde_json::Value::String(req_header.uri.path().to_string())
                );
                ctx.plugins_data.insert(
                    "performance_analyzer_method".to_string(),
                    serde_json::Value::String(req_header.method.to_string())
                );
                
                Ok((false, None))
            },
            PluginStep::ProxyUpstream => {
        // Mark the end of request processing and start of upstream request
                if let Some(perf_ctx_val) = ctx.plugins_data.get_mut("performance_analyzer_context") {
                    if let Ok(mut perf_ctx) = serde_json::from_value::<PerformancePluginContext>(perf_ctx_val.clone()) {
                        perf_ctx.request_processing_end_unix_ns = Some(PerformancePluginContext::now_ns());
                        perf_ctx.upstream_request_start_unix_ns = Some(PerformancePluginContext::now_ns());
                        *perf_ctx_val = serde_json::to_value(perf_ctx)?;
                    }
                }
                
                Ok((false, None))
            },
            _ => Ok((false, None))
        }
    }
    
    async fn handle_response(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
        upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        let config = self.config.lock().await.clone();
        
        if !config.enabled {
            return Ok(false);
        }
        
        match step {
            PluginStep::Response => {
                // Mark the time we received the response
                if let Some(perf_ctx_val) = ctx.plugins_data.get_mut("performance_analyzer_context") {
                    if let Ok(mut perf_ctx) = serde_json::from_value::<PerformancePluginContext>(perf_ctx_val.clone()) {
                        perf_ctx.upstream_response_received_unix_ns = Some(PerformancePluginContext::now_ns());
                        *perf_ctx_val = serde_json::to_value(perf_ctx)?;
                    }
                }
                
                // Add performance headers if configured
                if config.trace_headers {
                    // Add headers with performance timing data
                    if let Some(perf_ctx_val) = ctx.plugins_data.get("performance_analyzer_context") {
                        if let Ok(perf_ctx) = serde_json::from_value::<PerformancePluginContext>(perf_ctx_val.clone()) {
                            let req_processing_ms = PerformancePluginContext::duration_ms(
                                Some(perf_ctx.start_time_unix_ns),
                                perf_ctx.request_processing_end_unix_ns
                            );
                            let upstream_latency_ms = PerformancePluginContext::duration_ms(
                                perf_ctx.upstream_request_start_unix_ns,
                                perf_ctx.upstream_response_received_unix_ns
                            );
                            
                            upstream_response.insert_header("X-Request-Processing-Time", &req_processing_ms.to_string())?;
                            upstream_response.insert_header("X-Upstream-Latency", &upstream_latency_ms.to_string())?;
                        }
                    }
                }
                
                Ok(true)
            },
            PluginStep::Log => {
                // Finalize metrics for this request
                if let Err(e) = self.finalize_metrics(session, ctx, &config).await {
                    error!("Failed to finalize metrics: {}", e);
                }
                
                // Detect hotspots periodically
                if config.hotspot_detection {
                    if rand::thread_rng().gen_bool(0.01) { // Run occasionally (1% chance)
                        tokio::spawn(async move {
                            let analyzer = PerformanceAnalyzer::default();
                            if let Err(e) = analyzer.detect_hotspots().await {
                                error!("Failed to detect hotspots: {}", e);
                            }
                        });
                    }
                }
                
                Ok(false)
            },
            _ => Ok(false)
        }
    }
}

// Remove old test module
// #[cfg(test)]
// mod tests {
// ... tests ...
// } 