use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use std::any::Any;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bytes;
use chrono::{DateTime, Utc};
use http::StatusCode;
use pingora::{http::{RequestHeader, ResponseHeader}, proxy::Session};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{config::RoutePlugin, proxy_server::https_proxy::RouterContext};

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

    // Initialize timing context on each request
    fn init_timing(&self, _ctx: &mut RouterContext) {
        // Simplified implementation that doesn't use extensions
    }
    
    // Mark the end of request processing
    fn mark_request_processed(&self, _ctx: &mut RouterContext) {
        // Simplified implementation that doesn't use extensions
    }
    
    // Mark when upstream response is received
    fn mark_upstream_received(&self, _ctx: &mut RouterContext) {
        // Simplified implementation that doesn't use extensions
    }
    
    // Finalize and compute all metrics
    async fn finalize_metrics(&self, session: &mut Session, _ctx: &mut RouterContext, config: &PerformanceAnalyzerConfig) -> Result<()> {
        // Skip sampling if needed
        if rand::random::<f64>() > config.sample_rate {
            return Ok(());
        }
        
        // Simplified metrics collection - just use basic request info without timing
        let metrics = RequestMetrics {
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            provider: session.req_header().headers.get("x-llm-provider")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            model: session.req_header().headers.get("x-llm-model")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            request_size: session.req_header().headers
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0),
            response_size: None, // Can't access response header safely
            total_duration_ms: 0, // Placeholder
            request_processing_ms: 0, // Placeholder
            upstream_latency_ms: 0, // Placeholder
            response_processing_ms: 0, // Placeholder
            token_count_input: None,
            token_count_output: None,
            error: None,
            path: session.req_header().uri.path().to_string(),
            method: session.req_header().method.as_str().to_string(),
            status_code: None,
            client_ip: session.req_header().headers.get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            tags: HashMap::new(),
        };
        
        // Store metrics
        self.store_metrics(metrics, config).await?;
        
        Ok(())
    }

    async fn store_metrics(&self, metrics: RequestMetrics, config: &PerformanceAnalyzerConfig) -> Result<()> {
        // Process based on storage configuration
        match &config.storage {
            MetricsStorage::Memory { max_entries } => {
                // Store in memory with limit
                let mut metrics_lock = self.metrics.lock().await;
                metrics_lock.push(metrics);
                
                // Limit size if needed
                if metrics_lock.len() > *max_entries {
                    metrics_lock.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    metrics_lock.truncate(*max_entries);
                }
            },
            MetricsStorage::File { path: _ } => {
                // Simplified - just log instead of writing to file
                info!("Would store metric to file: {}", metrics.request_id);
            },
            MetricsStorage::Redis { url: _, key_prefix: _ } => {
                // Simplified - just log instead of using Redis
                info!("Would store metric to Redis: {}", metrics.request_id);
            },
            MetricsStorage::Prometheus { endpoint: _ } => {
                // Simplified - just log instead of using Prometheus
                info!("Would expose metric to Prometheus: {}", metrics.request_id);
            },
        }
        
        Ok(())
    }
    
    async fn detect_hotspots(&self) -> Result<()> {
        // Simplified hotspot detection
        info!("Running hotspot detection");
        Ok(())
    }

    // Serve the UI page 
    async fn serve_ui(&self, session: &mut Session, _ctx: &mut RouterContext) -> Result<bool> {
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
        session.write_response_body(Some(bytes::Bytes::from(html.into_bytes())), true).await?;

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

/* // Commented out outdated implementation
#[async_trait]
impl MiddlewarePlugin for PerformanceAnalyzer {
    // ... implementation ...
}
*/ 