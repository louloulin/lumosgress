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

use crate::{plugins::core::{Plugin, PluginError, PluginStep}, proxy_server::{https_proxy::RouterContext, HttpResponse}};

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
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(PerformanceAnalyzerConfig::default())),
            metrics: Arc::new(Mutex::new(Vec::new())),
            hot_spots: Arc::new(Mutex::new(Vec::new())),
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
            MetricsStorage::File { path } => {
                // TODO: Implement file storage (append to file, handle rotation)
                warn!("File storage for performance metrics not yet implemented.");
            }
            MetricsStorage::Redis { url, key_prefix } => {
                // TODO: Implement Redis storage (connect, store metric)
                 warn!("Redis storage for performance metrics not yet implemented.");
            }
            MetricsStorage::Prometheus { endpoint } => {
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

    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting performance analyzer plugin");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping performance analyzer plugin");
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