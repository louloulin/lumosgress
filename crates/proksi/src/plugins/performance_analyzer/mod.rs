use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use http::StatusCode;
use pingora::{http::{RequestHeader, ResponseHeader}, proxy::Session};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{config::RoutePlugin, proxy_server::https_proxy::RouterContext};
use super::MiddlewarePlugin;

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
        }
    }
}

// Timing data for a request stored in context
#[derive(Debug, Clone)]
pub struct RequestTimings {
    pub start_time: Instant,
    pub request_processing_end: Option<Instant>,
    pub upstream_request_start: Option<Instant>,
    pub upstream_response_received: Option<Instant>,
    pub response_processing_end: Option<Instant>,
}

// Holds data about the current performance trace
#[derive(Debug, Clone)]
pub struct PerformanceTrace {
    pub trace_id: String,
    pub spans: Vec<TraceSpan>,
    pub start_time: Instant,
    pub is_active: bool,
}

// A single span in a performance trace
#[derive(Debug, Clone)]
pub struct TraceSpan {
    pub span_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub tags: HashMap<String, String>,
}

// Main Performance Analyzer plugin
pub struct PerformanceAnalyzer {
    config: Arc<Mutex<HashMap<String, PerformanceAnalyzerConfig>>>,
    metrics: Arc<Mutex<Vec<RequestMetrics>>>,
    hot_spots: Arc<Mutex<Vec<HotSpot>>>,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(Vec::new())),
            hot_spots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Parse configuration from the plugin
    async fn parse_config(&self, plugin: &RoutePlugin) -> Result<PerformanceAnalyzerConfig> {
        if let Some(config) = &plugin.config {
            if let Some(config_name) = config.get("config_name") {
                if let Some(config_name) = config_name.as_str() {
                    let configs = self.config.lock().await;
                    if let Some(config) = configs.get(config_name) {
                        return Ok(config.clone());
                    }
                }
            }

            // Parse enabled flag
            let enabled = config
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            // Parse UI endpoint
            let ui_endpoint = config
                .get("ui_endpoint")
                .and_then(|v| v.as_str())
                .unwrap_or("/performance-analyzer")
                .to_string();

            // Parse sample rate
            let sample_rate = config
                .get("sample_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);

            // Parse detailed profiling flag
            let detailed_profiling = config
                .get("detailed_profiling")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Parse hotspot detection flag
            let hotspot_detection = config
                .get("hotspot_detection")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            // Parse token usage profiling flag
            let profile_token_usage = config
                .get("profile_token_usage")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            // Parse max trace depth
            let max_trace_depth = config
                .get("max_trace_depth")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);

            // Parse excluded paths
            let exclude_paths = config
                .get("exclude_paths")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_else(|| vec!["/health".to_string(), "/metrics".to_string()]);

            // Parse trace headers flag
            let trace_headers = config
                .get("trace_headers")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Parse storage configuration
            let storage = if let Some(storage_obj) = config.get("storage").and_then(|v| v.as_object()) {
                let storage_type = storage_obj.get("type").and_then(|v| v.as_str());

                match storage_type {
                    Some("file") => {
                        let path = storage_obj
                            .get("path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("./performance_metrics.json")
                            .to_string();
                        MetricsStorage::File { path }
                    }
                    Some("redis") => {
                        let url = storage_obj
                            .get("url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("redis://localhost:6379")
                            .to_string();
                        let key_prefix = storage_obj
                            .get("key_prefix")
                            .and_then(|v| v.as_str())
                            .unwrap_or("perf:")
                            .to_string();
                        MetricsStorage::Redis { url, key_prefix }
                    }
                    Some("prometheus") => {
                        let endpoint = storage_obj
                            .get("endpoint")
                            .and_then(|v| v.as_str())
                            .unwrap_or("/metrics")
                            .to_string();
                        MetricsStorage::Prometheus { endpoint }
                    }
                    _ => {
                        // Default to in-memory storage
                        let max_entries = storage_obj
                            .get("max_entries")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(10000) as usize;
                        MetricsStorage::Memory { max_entries }
                    }
                }
            } else {
                // Default to in-memory storage
                MetricsStorage::Memory { max_entries: 10000 }
            };

            return Ok(PerformanceAnalyzerConfig {
                enabled,
                ui_endpoint,
                sample_rate,
                detailed_profiling,
                storage,
                hotspot_detection,
                profile_token_usage,
                max_trace_depth,
                exclude_paths,
                trace_headers,
            });
        }

        // Return default configuration if no specific config provided
        Ok(PerformanceAnalyzerConfig::default())
    }

    // Initialize timing data in request context
    fn init_timing(&self, ctx: &mut RouterContext) {
        let timings = RequestTimings {
            start_time: Instant::now(),
            request_processing_end: None,
            upstream_request_start: None,
            upstream_response_received: None,
            response_processing_end: None,
        };

        // Store timings in extensions as a standard value, not a Box
        ctx.extensions.insert("performance_timings".to_string(), timings);
        
        if !ctx.extensions.contains_key("request_id") {
            let request_id = Uuid::new_v4().to_string();
            ctx.extensions.insert("request_id".to_string(), request_id);
        }
    }

    // Mark request processing end and upstream request start
    fn mark_request_processed(&self, ctx: &mut RouterContext) {
        if let Some(timings) = ctx.extensions.get_mut("performance_timings") {
            // In the updated API, extensions returns the raw value, not a Box
            if let Some(timings) = timings.downcast_mut::<RequestTimings>() {
                timings.request_processing_end = Some(Instant::now());
                timings.upstream_request_start = Some(Instant::now());
            }
        }
    }

    // Mark upstream response received
    fn mark_upstream_received(&self, ctx: &mut RouterContext) {
        if let Some(timings) = ctx.extensions.get_mut("performance_timings") {
            // In the updated API, extensions returns the raw value, not a Box
            if let Some(timings) = timings.downcast_mut::<RequestTimings>() {
                timings.upstream_response_received = Some(Instant::now());
            }
        }
    }

    // Mark response processing end and calculate metrics
    async fn finalize_metrics(&self, session: &mut Session, ctx: &mut RouterContext, config: &PerformanceAnalyzerConfig) -> Result<()> {
        // First mark the response processing as complete
        if let Some(timings) = ctx.extensions.get_mut("performance_timings") {
            if let Some(timings) = timings.downcast_mut::<RequestTimings>() {
                timings.response_processing_end = Some(Instant::now());
            }
        }

        // Decide whether to record metrics based on sampling rate
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() > config.sample_rate {
            return Ok(());
        }

        // Extract timing data
        if let Some(timings) = ctx.extensions.get("performance_timings") {
            if let Some(timings) = timings.downcast_ref::<RequestTimings>() {
                // Calculate durations
                let total_duration = timings.start_time.elapsed();
                
                let request_processing = if let Some(end) = timings.request_processing_end {
                    end.duration_since(timings.start_time)
                } else {
                    Duration::from_secs(0)
                };
                
                let upstream_latency = if let (Some(start), Some(end)) = (timings.upstream_request_start, timings.upstream_response_received) {
                    end.duration_since(start)
                } else {
                    Duration::from_secs(0)
                };
                
                let response_processing = if let (Some(start), Some(end)) = (timings.upstream_response_received, timings.response_processing_end) {
                    end.duration_since(start)
                } else {
                    Duration::from_secs(0)
                };

                // Extract request metadata
                let request_id = if let Some(id) = ctx.extensions.get("request_id") {
                    if let Some(id_str) = id.downcast_ref::<String>() {
                        id_str.clone()
                    } else {
                        Uuid::new_v4().to_string()
                    }
                } else {
                    Uuid::new_v4().to_string()
                };

                // Extract LLM provider and model if available
                let metadata = ctx.extensions.get("request_metadata")
                    .and_then(|m| m.downcast_ref::<HashMap<String, String>>())
                    .cloned()
                    .unwrap_or_default();
                
                let provider = metadata.get("llm_provider").cloned();
                let model = metadata.get("llm_model").cloned();

                // Extract token usage if available
                let token_count_input = if config.profile_token_usage {
                    metadata.get("token_count_input").and_then(|s| s.parse::<usize>().ok())
                } else {
                    None
                };
                
                let token_count_output = if config.profile_token_usage {
                    metadata.get("token_count_output").and_then(|s| s.parse::<usize>().ok())
                } else {
                    None
                };

                // Create metrics record
                let metrics = RequestMetrics {
                    request_id,
                    timestamp: Utc::now(),
                    provider,
                    model,
                    request_size: session.req_header().len_of_content().unwrap_or(0) as usize,
                    response_size: None, // We don't have this information readily available
                    total_duration_ms: total_duration.as_millis() as u64,
                    request_processing_ms: request_processing.as_millis() as u64,
                    upstream_latency_ms: upstream_latency.as_millis() as u64,
                    response_processing_ms: response_processing.as_millis() as u64,
                    token_count_input,
                    token_count_output,
                    error: None, // Would need to extract from response if available
                    path: session.req_header().uri().to_string(),
                    method: session.req_header().method().to_string(),
                    status_code: None, // Would need to extract from response
                    client_ip: None, // Would need to extract from request
                    tags: HashMap::new(),
                };

                // Store metrics
                self.store_metrics(metrics, config).await?;

                // Detect hot spots if enabled
                if config.hotspot_detection && upstream_latency.as_millis() > 500 {
                    self.detect_hotspots().await?;
                }
            }
        }

        Ok(())
    }

    // Store metrics according to the configured storage backend
    async fn store_metrics(&self, metrics: RequestMetrics, config: &PerformanceAnalyzerConfig) -> Result<()> {
        match &config.storage {
            MetricsStorage::Memory { max_entries } => {
                let mut metrics_storage = self.metrics.lock().await;
                metrics_storage.push(metrics);
                
                // Prune if needed
                if metrics_storage.len() > *max_entries {
                    metrics_storage.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    metrics_storage.truncate(*max_entries);
                }
            },
            MetricsStorage::File { path } => {
                // In a real implementation, this would append to a file
                // For simplicity, we'll just use the in-memory storage for now
                let mut metrics_storage = self.metrics.lock().await;
                metrics_storage.push(metrics);
            },
            MetricsStorage::Redis { url, key_prefix } => {
                // In a real implementation, this would use Redis
                // For simplicity, we'll just use the in-memory storage for now
                let mut metrics_storage = self.metrics.lock().await;
                metrics_storage.push(metrics);
            },
            MetricsStorage::Prometheus { endpoint } => {
                // In a real implementation, this would expose Prometheus metrics
                // For simplicity, we'll just use the in-memory storage for now
                let mut metrics_storage = self.metrics.lock().await;
                metrics_storage.push(metrics);
            },
        }

        Ok(())
    }

    // Detect performance hot spots
    async fn detect_hotspots(&self) -> Result<()> {
        let metrics = self.metrics.lock().await;
        let mut hot_spots = self.hot_spots.lock().await;
        
        // This would implement proper hot spot detection algorithms
        // For now, we just have a placeholder implementation

        // Example: Check for slow upstream requests
        let slow_requests_count = metrics.iter()
            .filter(|m| m.upstream_latency_ms > 500)
            .count();
            
        if slow_requests_count > 10 {
            // Example: Identify a potential hot spot
            let avg_time = metrics.iter()
                .filter(|m| m.upstream_latency_ms > 500)
                .map(|m| m.upstream_latency_ms as f64)
                .sum::<f64>() / slow_requests_count as f64;
            
            // Add a hot spot if we don't already have one for slow upstream requests
            if !hot_spots.iter().any(|h| h.name == "Slow Upstream Requests") {
                hot_spots.push(HotSpot {
                    id: Uuid::new_v4().to_string(),
                    name: "Slow Upstream Requests".to_string(),
                    description: "High latency detected in upstream responses".to_string(),
                    impact_level: if avg_time > 2000.0 { 
                        HotSpotImpact::High 
                    } else { 
                        HotSpotImpact::Medium 
                    },
                    avg_time_ms: avg_time,
                    occurrence_count: slow_requests_count,
                    recommendation: "Consider scaling up upstream services or optimizing response time".to_string(),
                    identified_at: Utc::now(),
                });
            }
        }

        Ok(())
    }

    // Serve the UI
    async fn serve_ui(&self, session: &mut Session, ctx: &mut RouterContext) -> Result<bool> {
        // For actual implementation, this would render a proper UI
        // For now, we'll just return a simple HTML page
        let metrics = self.metrics.lock().await;
        let hot_spots = self.hot_spots.lock().await;
        
        let html = format!(
            r#"<!DOCTYPE html>
            <html>
            <head>
                <title>Performance Analyzer</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}
                    h1, h2 {{ color: #333; }}
                    .dashboard {{ display: flex; flex-wrap: wrap; }}
                    .card {{ background: #f5f5f5; border-radius: 5px; padding: 15px; margin: 10px; flex: 1 1 300px; }}
                    .metrics {{ width: 100%; border-collapse: collapse; }}
                    .metrics th, .metrics td {{ text-align: left; padding: 8px; border-bottom: 1px solid #ddd; }}
                    .hotspot {{ background: #fff8e1; border-left: 4px solid #ffc107; padding: 10px; margin: 10px 0; }}
                    .hotspot.high {{ background: #ffebee; border-left: 4px solid #f44336; }}
                    .hotspot.critical {{ background: #ffebee; border-left: 4px solid #d50000; }}
                </style>
            </head>
            <body>
                <h1>Performance Analyzer Dashboard</h1>
                <div class="dashboard">
                    <div class="card">
                        <h2>Request Statistics</h2>
                        <p>Total Requests: {}</p>
                        <p>Average Response Time: {:.2} ms</p>
                        <p>Average Upstream Latency: {:.2} ms</p>
                    </div>
                </div>
                <h2>Performance Hot Spots</h2>
                <div class="hotspots">
                    {}
                </div>
                <h2>Recent Requests</h2>
                <table class="metrics">
                    <tr>
                        <th>Time</th>
                        <th>Path</th>
                        <th>Provider</th>
                        <th>Model</th>
                        <th>Total Time (ms)</th>
                        <th>Upstream Latency (ms)</th>
                    </tr>
                    {}
                </table>
            </body>
            </html>"#,
            metrics.len(),
            metrics.iter().map(|m| m.total_duration_ms as f64).sum::<f64>() / metrics.len().max(1) as f64,
            metrics.iter().map(|m| m.upstream_latency_ms as f64).sum::<f64>() / metrics.len().max(1) as f64,
            hot_spots.iter().map(|h| {
                let severity_class = match h.impact_level {
                    HotSpotImpact::Low => "",
                    HotSpotImpact::Medium => "",
                    HotSpotImpact::High => "high",
                    HotSpotImpact::Critical => "critical",
                };
                format!(
                    r#"<div class="hotspot {}">
                        <h3>{}</h3>
                        <p>{}</p>
                        <p><strong>Average Time:</strong> {:.2} ms</p>
                        <p><strong>Occurrences:</strong> {}</p>
                        <p><strong>Recommendation:</strong> {}</p>
                    </div>"#,
                    severity_class, h.name, h.description, h.avg_time_ms, h.occurrence_count, h.recommendation
                )
            }).collect::<Vec<_>>().join("\n"),
            metrics.iter().take(100).map(|m| {
                format!(
                    r#"<tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                    </tr>"#,
                    m.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    m.path,
                    m.provider.as_deref().unwrap_or("-"),
                    m.model.as_deref().unwrap_or("-"),
                    m.total_duration_ms,
                    m.upstream_latency_ms
                )
            }).collect::<Vec<_>>().join("\n")
        );

        // Write HTTP response
        let content_length = html.len();
        let mut response = ResponseHeader::build(StatusCode::OK, None)?;
        response.append_header("Content-Type", "text/html; charset=utf-8")?;
        response.append_header("Content-Length", content_length.to_string())?;

        session.write_response_header(Box::new(response), false).await?;
        session.write_response_body(Some(html.into_bytes()), true).await?;

        Ok(true)
    }

    // Generate metrics summary
    fn generate_summary(&self, metrics: &[RequestMetrics]) -> MetricsSummary {
        if metrics.is_empty() {
            return MetricsSummary {
                avg_response_time_ms: 0.0,
                p50_response_time_ms: 0.0,
                p90_response_time_ms: 0.0,
                p95_response_time_ms: 0.0,
                p99_response_time_ms: 0.0,
                avg_upstream_latency_ms: 0.0,
                avg_request_size: 0.0,
                avg_response_size: 0.0,
                avg_tokens_input: 0.0,
                avg_tokens_output: 0.0,
                success_rate: 100.0,
                request_count: 0,
                error_count: 0,
                period_start: Utc::now(),
                period_end: Utc::now(),
            };
        }

        // Calculate response times for percentiles
        let mut response_times: Vec<u64> = metrics.iter()
            .map(|m| m.total_duration_ms)
            .collect();
        response_times.sort();

        let len = response_times.len();
        let p50_idx = (len as f64 * 0.5) as usize;
        let p90_idx = (len as f64 * 0.9) as usize;
        let p95_idx = (len as f64 * 0.95) as usize;
        let p99_idx = (len as f64 * 0.99) as usize;

        // Calculate averages
        let avg_response_time = metrics.iter()
            .map(|m| m.total_duration_ms as f64)
            .sum::<f64>() / len as f64;
        
        let avg_upstream_latency = metrics.iter()
            .map(|m| m.upstream_latency_ms as f64)
            .sum::<f64>() / len as f64;
        
        let avg_request_size = metrics.iter()
            .map(|m| m.request_size as f64)
            .sum::<f64>() / len as f64;
        
        let response_sizes: Vec<f64> = metrics.iter()
            .filter_map(|m| m.response_size.map(|s| s as f64))
            .collect();
        let avg_response_size = if !response_sizes.is_empty() {
            response_sizes.iter().sum::<f64>() / response_sizes.len() as f64
        } else {
            0.0
        };
        
        let token_inputs: Vec<f64> = metrics.iter()
            .filter_map(|m| m.token_count_input.map(|t| t as f64))
            .collect();
        let avg_tokens_input = if !token_inputs.is_empty() {
            token_inputs.iter().sum::<f64>() / token_inputs.len() as f64
        } else {
            0.0
        };
        
        let token_outputs: Vec<f64> = metrics.iter()
            .filter_map(|m| m.token_count_output.map(|t| t as f64))
            .collect();
        let avg_tokens_output = if !token_outputs.is_empty() {
            token_outputs.iter().sum::<f64>() / token_outputs.len() as f64
        } else {
            0.0
        };
        
        // Calculate error rate
        let error_count = metrics.iter()
            .filter(|m| m.error.is_some())
            .count();
        let success_rate = 100.0 * (len - error_count) as f64 / len as f64;
        
        // Find period start and end
        let period_start = metrics.iter()
            .map(|m| m.timestamp)
            .min()
            .unwrap_or_else(Utc::now);
        let period_end = metrics.iter()
            .map(|m| m.timestamp)
            .max()
            .unwrap_or_else(Utc::now);

        MetricsSummary {
            avg_response_time_ms: avg_response_time,
            p50_response_time_ms: response_times.get(p50_idx).copied().unwrap_or(0) as f64,
            p90_response_time_ms: response_times.get(p90_idx).copied().unwrap_or(0) as f64,
            p95_response_time_ms: response_times.get(p95_idx).copied().unwrap_or(0) as f64,
            p99_response_time_ms: response_times.get(p99_idx).copied().unwrap_or(0) as f64,
            avg_upstream_latency_ms: avg_upstream_latency,
            avg_request_size,
            avg_response_size,
            avg_tokens_input,
            avg_tokens_output,
            success_rate,
            request_count: len,
            error_count,
            period_start,
            period_end,
        }
    }
}

#[async_trait]
impl MiddlewarePlugin for PerformanceAnalyzer {
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut RouterContext,
        plugin: &RoutePlugin,
    ) -> Result<bool> {
        // Parse configuration
        let config = self.parse_config(plugin).await?;
        
        // Check if plugin is enabled
        if !config.enabled {
            return Ok(false);
        }
        
        // Check if this is a request to the UI endpoint
        let request_path = session.req_header().uri().to_string();
        if request_path == config.ui_endpoint {
            return self.serve_ui(session, ctx).await;
        }
        
        // Check if this path should be excluded
        if config.exclude_paths.contains(&request_path) {
            return Ok(false);
        }
        
        // Initialize timing data
        self.init_timing(ctx);
        
        // Continue processing the request
        Ok(false)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        _upstream_request: &mut RequestHeader,
        ctx: &mut RouterContext,
    ) -> Result<()> {
        // Mark the end of request processing and start of upstream request
        self.mark_request_processed(ctx);
        
        Ok(())
    }

    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        _upstream_response: &mut ResponseHeader,
        ctx: &mut RouterContext,
    ) -> Result<()> {
        // Mark the receipt of upstream response
        self.mark_upstream_received(ctx);
        
        Ok(())
    }
    
    async fn response_filter(
        &self,
        session: &mut Session,
        ctx: &mut RouterContext,
        plugin: &RoutePlugin,
    ) -> Result<bool> {
        // Parse configuration
        let config = self.parse_config(plugin).await?;
        
        // Check if plugin is enabled
        if !config.enabled {
            return Ok(false);
        }
        
        // Check if this path should be excluded
        let request_path = session.req_header().uri().to_string();
        if config.exclude_paths.contains(&request_path) {
            return Ok(false);
        }
        
        // Finalize metrics collection
        self.finalize_metrics(session, ctx, &config).await?;
        
        // Continue processing the response
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::borrow::Cow;

    #[tokio::test]
    async fn test_default_config() {
        let config = PerformanceAnalyzerConfig::default();
        
        // Verify default values
        assert!(config.enabled);
        assert_eq!(config.ui_endpoint, "/performance-analyzer");
        assert_eq!(config.sample_rate, 1.0);
        assert!(!config.detailed_profiling);
        assert!(config.hotspot_detection);
        assert!(config.profile_token_usage);
    }

    #[tokio::test]
    async fn test_generate_summary() {
        let analyzer = PerformanceAnalyzer::new();
        
        // Create some sample metrics
        let metrics = vec![
            RequestMetrics {
                request_id: "1".to_string(),
                timestamp: Utc::now(),
                provider: Some("openai".to_string()),
                model: Some("gpt-4".to_string()),
                request_size: 100,
                response_size: Some(200),
                total_duration_ms: 150,
                request_processing_ms: 20,
                upstream_latency_ms: 120,
                response_processing_ms: 10,
                token_count_input: Some(50),
                token_count_output: Some(100),
                error: None,
                path: "/api/completion".to_string(),
                method: "POST".to_string(),
                status_code: Some(200),
                client_ip: Some("127.0.0.1".to_string()),
                tags: HashMap::new(),
            },
            RequestMetrics {
                request_id: "2".to_string(),
                timestamp: Utc::now(),
                provider: Some("anthropic".to_string()),
                model: Some("claude-2".to_string()),
                request_size: 150,
                response_size: Some(300),
                total_duration_ms: 250,
                request_processing_ms: 30,
                upstream_latency_ms: 200,
                response_processing_ms: 20,
                token_count_input: Some(75),
                token_count_output: Some(150),
                error: None,
                path: "/api/completion".to_string(),
                method: "POST".to_string(),
                status_code: Some(200),
                client_ip: Some("127.0.0.1".to_string()),
                tags: HashMap::new(),
            },
        ];
        
        // Generate summary
        let summary = analyzer.generate_summary(&metrics);
        
        // Verify summary values
        assert_eq!(summary.request_count, 2);
        assert_eq!(summary.error_count, 0);
        assert_eq!(summary.success_rate, 100.0);
        assert_eq!(summary.avg_response_time_ms, 200.0); // (150 + 250) / 2
        assert_eq!(summary.avg_upstream_latency_ms, 160.0); // (120 + 200) / 2
        assert_eq!(summary.avg_tokens_input, 62.5); // (50 + 75) / 2
    }
} 