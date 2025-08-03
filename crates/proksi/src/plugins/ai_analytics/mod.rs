use std::sync::Arc;
use anyhow::Result;
use pingora::proxy::Session;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use pingora::http::ResponseHeader;
use std::fmt::Debug;

use crate::proxy_server::https_proxy::RouterContext;

// Import new Plugin trait and related types
use crate::plugins::core::{Plugin, PluginError, PluginStep};
use crate::proxy_server::HttpResponse;

mod anomaly_detection;

pub use anomaly_detection::{
    AnomalyDetectionConfig, AnomalyDetectionType, 
    AnomalyAlert, AnomalySeverity, AlertChannelConfig, AlertChannelType
};
use anomaly_detection::AnomalyDetector;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiAnalyticsConfig {
    pub metrics_enabled: bool,
    pub storage_type: AnalyticsStorageType,
    pub retention_days: u32,
    pub sampling_rate: f64,
    pub alert_thresholds: HashMap<String, f64>,
    pub dashboard_enabled: bool,
    pub dashboard_path: Option<String>,
    pub anomaly_detection: Option<AnomalyDetectionConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum AnalyticsStorageType {
    #[default]
    InMemory,
    Redis,
    Postgres,
    Custom(String),
}

impl AnalyticsStorageType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "inmemory" => Some(Self::InMemory),
            "redis" => Some(Self::Redis),
            "postgres" => Some(Self::Postgres),
            _ => Some(Self::Custom(s.to_string())),
        }
    }
}

impl FromStr for AnalyticsStorageType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AnalyticsStorageType::from_str(s)
            .ok_or_else(|| anyhow::anyhow!("Invalid storage type: {}", s))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetric {
    pub timestamp: DateTime<Utc>,
    pub metric_name: String,
    pub value: f64,
    pub tags: HashMap<String, String>,
}

#[derive(Debug)]
pub struct AiAnalytics {
    config: AiAnalyticsConfig,
    storage: Option<Arc<Mutex<dyn AnalyticsStorage>>>,
    metrics: Arc<Mutex<HashMap<String, Vec<AnalyticsMetric>>>>,
    anomaly_detector: Option<Arc<Mutex<AnomalyDetector>>>,
    last_prune_time: Arc<Mutex<DateTime<Utc>>>,
}

#[async_trait]
trait AnalyticsStorage: Send + Sync + Debug {
    async fn store_metric(&mut self, metric: AnalyticsMetric) -> Result<()>;
    #[allow(dead_code)]
    async fn get_metrics(&self, metric_name: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<AnalyticsMetric>>;
    #[allow(dead_code)]
    async fn get_aggregated_metrics(&self, metric_name: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>, aggregation: AggregationType) -> Result<Vec<AnalyticsMetric>>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregationType {
    Average,
    Sum,
    Max,
    Min,
    Count,
}

impl AiAnalytics {
    pub fn new() -> Self {
        Self {
            config: AiAnalyticsConfig::default(),
            storage: None,
            metrics: Arc::new(Mutex::new(HashMap::new())),
            anomaly_detector: None,
            last_prune_time: Arc::new(Mutex::new(Utc::now())),
        }
    }

    pub fn with_config(config: AiAnalyticsConfig) -> Self {
        Self {
            config,
            storage: None,
            metrics: Arc::new(Mutex::new(HashMap::new())),
            anomaly_detector: None,
            last_prune_time: Arc::new(Mutex::new(Utc::now())),
        }
    }

    async fn initialize_storage_and_detector(&mut self) -> Result<()> {
        let config = &self.config;
        
        let storage_instance: Arc<Mutex<dyn AnalyticsStorage>> = match config.storage_type {
            AnalyticsStorageType::InMemory => Arc::new(Mutex::new(InMemoryStorage::new())),
            AnalyticsStorageType::Redis => Arc::new(Mutex::new(RedisStorage::new(config))),
            AnalyticsStorageType::Postgres => Arc::new(Mutex::new(PostgresStorage::new(config))),
            AnalyticsStorageType::Custom(_) => Arc::new(Mutex::new(CustomStorage::new(config))),
        };
        self.storage = Some(storage_instance);
        
        if let Some(anomaly_config) = &config.anomaly_detection {
            let detector = AnomalyDetector::new(anomaly_config.clone());
            self.anomaly_detector = Some(Arc::new(Mutex::new(detector)));
        }
        
        Ok(())
    }

    async fn collect_request_metrics(&self, session: &Session, ctx: &RouterContext) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();
        let timestamp = Utc::now();
        let mut tags = HashMap::new();

        // Basic request tags
        tags.insert("host".to_string(), ctx.host.clone());
        if let Some(path) = session.req_header().uri.path_and_query() {
             tags.insert("path".to_string(), path.path().to_string());
        }
        tags.insert("method".to_string(), session.req_header().method.as_str().to_string());
        tags.insert("request_id".to_string(), ctx.request_id.clone()); // Use request_id from ctx

        // Request Latency - Placeholder
        // NOTE: Cannot access private ctx.timings.request_filter_start. Using 0.0.
        // Accurate timing requires changes to RouterTimings or context.
        let req_latency = 0.0; 
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "request.latency".to_string(),
            value: req_latency,
            tags: tags.clone(),
        });

        // Request size
        let request_size = session.req_header().headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
            
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "request.size".to_string(),
            value: request_size,
            tags: tags.clone(),
        });

        // LLM provider metrics (from RouterContext if LlmRouter ran)
        if let Some(provider_val) = ctx.plugins_data.get("llm_provider") {
             if let Some(provider_str) = provider_val.as_str() {
                let mut provider_tags = tags.clone();
                provider_tags.insert("provider".to_string(), provider_str.to_string());
                
                metrics.push(AnalyticsMetric {
                    timestamp,
                    metric_name: "llm.provider.requests".to_string(),
                    value: 1.0,
                    tags: provider_tags,
                });
            }
        }
        
        // LLM model metrics (if available in context)
        // This depends on other plugins potentially adding this info
        if let Some(model_val) = ctx.plugins_data.get("llm_model") {
            if let Some(model_str) = model_val.as_str() {
                let mut model_tags = tags.clone();
                model_tags.insert("model".to_string(), model_str.to_string());
                
                metrics.push(AnalyticsMetric {
                    timestamp,
                    metric_name: "llm.model.requests".to_string(),
                    value: 1.0,
                    tags: model_tags,
                });
            }
        }

        Ok(metrics)
    }

    async fn collect_response_metrics(&self, _session: &Session, ctx: &RouterContext, upstream_response: &ResponseHeader) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();
        let timestamp = Utc::now();
        let mut tags = HashMap::new();
        
        // Basic tags (reuse from request context or re-extract)
        tags.insert("host".to_string(), ctx.host.clone());
        tags.insert("request_id".to_string(), ctx.request_id.clone()); 
        if let Some(provider_val) = ctx.plugins_data.get("llm_provider") {
             if let Some(provider_str) = provider_val.as_str() {
                tags.insert("provider".to_string(), provider_str.to_string());
             }
        }
        if let Some(model_val) = ctx.plugins_data.get("llm_model") {
            if let Some(model_str) = model_val.as_str() {
                tags.insert("model".to_string(), model_str.to_string());
            }
        }

        // Response Latency - Placeholder
        // NOTE: Cannot access private ctx.timings.request_filter_start. Using 0.0.
        // Accurate timing requires changes to RouterTimings or context.
        let total_latency = 0.0;
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "response.latency".to_string(),
            value: total_latency,
            tags: tags.clone(),
        });

        // Response Status Code
        let status = upstream_response.status.as_u16();
        tags.insert("status_code".to_string(), status.to_string());
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "response.status".to_string(),
            value: status as f64,
            tags: tags.clone(),
        });
        
        // Error count
        if status >= 400 {
            metrics.push(AnalyticsMetric {
                timestamp,
                metric_name: "response.errors".to_string(),
                value: 1.0,
                tags: tags.clone(),
            });
        }

        // Response size
        let response_size = upstream_response.headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
            
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "response.size".to_string(),
            value: response_size,
            tags: tags.clone(),
        });
        
        // Placeholder for Token Usage - needs body access/parsing
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "llm.token_usage".to_string(),
            value: 0.0, // Placeholder
            tags: tags.clone(),
        });

        Ok(metrics)
    }

    async fn check_alerts(&self) -> Result<()> {
        let config = &self.config;
        if config.alert_thresholds.is_empty() {
            return Ok(());
        }
        
        let now = Utc::now();
        let start_time = now - chrono::Duration::minutes(5); // Example window

        let metrics_map = self.metrics.lock().await;
        for (metric_name, threshold) in &config.alert_thresholds {
            let metrics = metrics_map
                .get(metric_name)
                .map(|metrics| {
                    metrics.iter()
                        .filter(|m| m.timestamp >= start_time)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if !metrics.is_empty() {
                let avg = metrics.iter().map(|m| m.value).sum::<f64>() / metrics.len() as f64;
                if avg > *threshold {
                    warn!(
                        "Alert triggered for metric {}: value {} exceeds threshold {}",
                        metric_name, avg, threshold
                    );
                    // TODO: Implement actual alerting mechanism (e.g., call webhook)
                }
            }
        }
        Ok(())
    }
    
    async fn prune_metric_history(&self) -> Result<()> {
        let config = &self.config;
        let retention_days = config.retention_days;
        let cutoff = Utc::now() - Duration::days(retention_days as i64);
        
        // Prune in-memory metrics cache
        let mut metrics_cache = self.metrics.lock().await;
        for values in metrics_cache.values_mut() {
            values.retain(|m| m.timestamp >= cutoff);
        }
        metrics_cache.retain(|_, v| !v.is_empty()); // Remove empty keys
        drop(metrics_cache);
        
        // Prune anomaly detector history
        if let Some(detector) = self.anomaly_detector.as_ref() {
            let mut detector_lock = detector.lock().await;
            detector_lock.prune_history(Duration::days(retention_days as i64));
        }
        
        // Update last prune time
        *self.last_prune_time.lock().await = Utc::now();
        
        Ok(())
    }
}

#[async_trait]
impl Plugin for AiAnalytics {
    fn name(&self) -> &'static str {
        "ai_analytics"
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        if step != PluginStep::Request || !self.config.metrics_enabled {
            return Ok((false, None));
        }
        // Sampling
        if rand::random::<f64>() > self.config.sampling_rate {
            return Ok((false, None));
        }
        
        match self.collect_request_metrics(session, ctx).await {
            Ok(metrics) => {
                if !metrics.is_empty() {
                    let mut metrics_map = self.metrics.lock().await;
                    for metric in metrics {
                        metrics_map.entry(metric.metric_name.clone())
                            .or_insert_with(Vec::new)
                            .push(metric);
                    }
                }
            }
            Err(e) => {
                error!("AiAnalytics: Failed to collect request metrics: {}", e);
            }
        }
        
        Ok((false, None))
    }

    async fn handle_response(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
        upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
         if step != PluginStep::Response || !self.config.metrics_enabled {
            return Ok(false);
        }
        // Sampling
        if rand::random::<f64>() > self.config.sampling_rate {
            return Ok(false);
        }
        
        let mut modified = false;
        match self.collect_response_metrics(session, ctx, upstream_response).await {
             Ok(metrics) => {
                if !metrics.is_empty() {
                    // Store metrics (assuming storage was initialized in start)
                    if let Some(storage) = self.storage.as_ref() {
                        let mut storage_lock = storage.lock().await;
                        for metric in metrics.clone() { // Clone metrics for storage & anomaly detection
                            if let Err(e) = storage_lock.store_metric(metric).await {
                                error!("AiAnalytics: Failed to store metric: {}", e);
                            }
                        }
                    } else {
                         warn!("AiAnalytics: Storage not initialized, cannot store metrics.");
                    }
                    
                    // Feed metrics to anomaly detector
                    if let Some(detector) = self.anomaly_detector.as_ref() {
                        let mut detector_lock = detector.lock().await;
                        for metric in &metrics {
                            detector_lock.add_metric(metric.clone());
                        }
                        let anomalies = detector_lock.detect_anomalies();
                        if !anomalies.is_empty() {
                            info!("Detected {} anomalies in metrics", anomalies.len());
                            // TODO: Handle anomalies (e.g., trigger alerts)
                        }
                    }
                    
                    // Add header
                    upstream_response.insert_header("X-Analytics-Processed", "true")?;
                    modified = true;
                }
            }
            Err(e) => {
                error!("AiAnalytics: Failed to collect response metrics: {}", e);
            }
        }
        
        // Check alerts (might be better done periodically in background)
        if let Err(e) = self.check_alerts().await {
             error!("AiAnalytics: Failed to check alerts: {}", e);
        }
        
        // Prune history (might be better done periodically in background)
        if Utc::now() - *self.last_prune_time.lock().await > Duration::hours(1) {
            if let Err(e) = self.prune_metric_history().await {
                error!("AiAnalytics: Failed to prune metrics: {}", e);
            }
        }

        Ok(modified)
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting AiAnalytics plugin...");
        self.initialize_storage_and_detector().await
            .map_err(|e| PluginError::InitializationFailed(format!("Failed to initialize storage/detector: {}", e)))?;
        info!("AiAnalytics plugin started successfully.");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping AiAnalytics plugin...");
        // Potential cleanup logic for storage connections if needed
        Ok(())
    }
}

#[derive(Debug)]
struct InMemoryStorage {
    metrics: HashMap<String, Vec<AnalyticsMetric>>,
}

impl InMemoryStorage {
    fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }
}

#[async_trait]
impl AnalyticsStorage for InMemoryStorage {
    async fn store_metric(&mut self, metric: AnalyticsMetric) -> Result<()> {
        self.metrics.entry(metric.metric_name.clone())
            .or_insert_with(Vec::new)
            .push(metric);
        Ok(())
    }

    async fn get_metrics(&self, metric_name: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<AnalyticsMetric>> {
        let filtered_metrics = self.metrics.get(metric_name)
            .map(|m| m.iter()
                .filter(|metric| metric.timestamp >= start_time && metric.timestamp <= end_time)
                .cloned()
                .collect())
            .unwrap_or_default();
        Ok(filtered_metrics)
    }

    async fn get_aggregated_metrics(&self, metric_name: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>, aggregation: AggregationType) -> Result<Vec<AnalyticsMetric>> {
        let metrics = self.get_metrics(metric_name, start_time, end_time).await?;
        let aggregated = match aggregation {
            AggregationType::Average => {
                let sum: f64 = metrics.iter().map(|m| m.value).sum();
                let count = metrics.len() as f64;
                vec![AnalyticsMetric {
                    timestamp: end_time,
                    metric_name: metric_name.to_string(),
                    value: if count > 0.0 { sum / count } else { 0.0 },
                    tags: HashMap::new(),
                }]
            },
            AggregationType::Sum => {
                let sum: f64 = metrics.iter().map(|m| m.value).sum();
                vec![AnalyticsMetric {
                    timestamp: end_time,
                    metric_name: metric_name.to_string(),
                    value: sum,
                    tags: HashMap::new(),
                }]
            },
            AggregationType::Max => {
                let max = metrics.iter().map(|m| m.value).fold(f64::MIN, f64::max);
                vec![AnalyticsMetric {
                    timestamp: end_time,
                    metric_name: metric_name.to_string(),
                    value: max,
                    tags: HashMap::new(),
                }]
            },
            AggregationType::Min => {
                let min = metrics.iter().map(|m| m.value).fold(f64::MAX, f64::min);
                vec![AnalyticsMetric {
                    timestamp: end_time,
                    metric_name: metric_name.to_string(),
                    value: min,
                    tags: HashMap::new(),
                }]
            },
            AggregationType::Count => {
                vec![AnalyticsMetric {
                    timestamp: end_time,
                    metric_name: metric_name.to_string(),
                    value: metrics.len() as f64,
                    tags: HashMap::new(),
                }]
            },
        };
        Ok(aggregated)
    }
}

#[derive(Debug)]
struct RedisStorage {
    #[allow(dead_code)]
    config: AiAnalyticsConfig,
}

impl RedisStorage {
    fn new(config: &AiAnalyticsConfig) -> Self {
        Self { config: config.clone() }
    }
}

#[async_trait]
impl AnalyticsStorage for RedisStorage {
    async fn store_metric(&mut self, _metric: AnalyticsMetric) -> Result<()> {
        Ok(())
    }

    async fn get_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>) -> Result<Vec<AnalyticsMetric>> {
        Ok(vec![])
    }

    async fn get_aggregated_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>, _aggregation: AggregationType) -> Result<Vec<AnalyticsMetric>> {
        Ok(vec![])
    }
}

#[derive(Debug)]
struct PostgresStorage {
    #[allow(dead_code)]
    config: AiAnalyticsConfig,
}

impl PostgresStorage {
    fn new(config: &AiAnalyticsConfig) -> Self {
        Self { config: config.clone() }
    }
}

#[async_trait]
impl AnalyticsStorage for PostgresStorage {
    async fn store_metric(&mut self, _metric: AnalyticsMetric) -> Result<()> {
        Ok(())
    }

    async fn get_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>) -> Result<Vec<AnalyticsMetric>> {
        Ok(vec![])
    }

    async fn get_aggregated_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>, _aggregation: AggregationType) -> Result<Vec<AnalyticsMetric>> {
        Ok(vec![])
    }
}

#[derive(Debug)]
struct CustomStorage {
    #[allow(dead_code)]
    config: AiAnalyticsConfig,
}

impl CustomStorage {
    fn new(config: &AiAnalyticsConfig) -> Self {
        Self { config: config.clone() }
    }
}

#[async_trait]
impl AnalyticsStorage for CustomStorage {
    async fn store_metric(&mut self, _metric: AnalyticsMetric) -> Result<()> {
        Ok(())
    }

    async fn get_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>) -> Result<Vec<AnalyticsMetric>> {
        Ok(vec![])
    }

    async fn get_aggregated_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>, _aggregation: AggregationType) -> Result<Vec<AnalyticsMetric>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use futures_util::FutureExt;

    #[test]
    fn test_anomaly_detection_config() {
        let config = AnomalyDetectionConfig {
            monitored_metrics: vec!["latency".to_string(), "tokens_per_request".to_string()],
            detection_interval_seconds: 300,
            detection_type: AnomalyDetectionType::ZScore(2.0),
            min_data_points: 30,
            tag_filters: Some(HashMap::from([
                ("model".to_string(), "gpt-4".to_string())
            ])),
            alert_suppression_window: Some(1800),
            alert_channel: AlertChannelConfig {
                channel_type: AlertChannelType::Slack,
                config: HashMap::from([
                    ("webhook_url".to_string(), "https://hooks.slack.com/...".to_string())
                ]),
            },
        };

        assert_eq!(config.monitored_metrics.len(), 2);
        assert_eq!(config.detection_interval_seconds, 300);
        assert_eq!(config.detection_type, AnomalyDetectionType::ZScore(2.0));
        assert_eq!(config.min_data_points, 30);
        assert!(config.tag_filters.is_some());
        assert_eq!(config.alert_suppression_window.unwrap(), 1800);
        assert!(matches!(config.alert_channel.channel_type, AlertChannelType::Slack));
    }

    #[tokio::test]
    async fn test_request_handling() {
        let analytics = AiAnalytics::new();
        let mut ctx = RouterContext {
            host: "example.com".to_string(),
            route_container: Default::default(),
            upstream: Default::default(),
            extensions: HashMap::new(),
            is_websocket: false,
            timings: Default::default(),
            plugins_data: HashMap::new(),
            request_id: "test-id".to_string(),
            upstream_response: None,
        };

        let headers = [""].join("\r\n");
        let input_header = format!("POST /api/chat HTTP/1.1\r\n{headers}\r\n\r\n");
        let mock_io = tokio_test::io::Builder::new().read(input_header.as_bytes()).build();
        let mut session = Session::new_h1(Box::new(mock_io));
        session.read_request().now_or_never().unwrap().unwrap();

        let result = analytics.handle_request(
            PluginStep::Request,
            &mut session,
            &mut ctx
        ).await;

        assert!(result.is_ok());
        let (handled, response) = result.unwrap();
        assert!(!handled);
        assert!(response.is_none());
    }
}