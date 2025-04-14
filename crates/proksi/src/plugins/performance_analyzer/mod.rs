use std::{borrow::Cow, collections::HashMap, sync::Arc, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
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
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{config::RoutePlugin, plugins::core::{Plugin, PluginError, PluginStep}, proxy_server::{https_proxy::RouterContext, HttpResponse}};

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

fn parse_plugin_config(plugin_config: Option<&RoutePlugin>) -> Result<PerformanceAnalyzerConfig> {
     let config_value = plugin_config
        .and_then(|p| p.config.as_ref())
        .ok_or_else(|| anyhow!("Missing PerformanceAnalyzer plugin configuration, using default."))?;

     // Parse enabled flag
    let enabled = config_value
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| PerformanceAnalyzerConfig::default().enabled);

    // Parse UI endpoint
    let ui_endpoint = config_value
        .get("ui_endpoint")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| PerformanceAnalyzerConfig::default().ui_endpoint);

    // Parse sample rate
    let sample_rate = config_value
        .get("sample_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| PerformanceAnalyzerConfig::default().sample_rate)
        .clamp(0.0, 1.0); // Ensure rate is between 0.0 and 1.0

    // Parse detailed profiling flag
    let detailed_profiling = config_value
        .get("detailed_profiling")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| PerformanceAnalyzerConfig::default().detailed_profiling);

    // Parse hotspot detection flag
    let hotspot_detection = config_value
        .get("hotspot_detection")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| PerformanceAnalyzerConfig::default().hotspot_detection);

    // Parse token usage profiling flag
    let profile_token_usage = config_value
        .get("profile_token_usage")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| PerformanceAnalyzerConfig::default().profile_token_usage);

    // Parse max trace depth
    let max_trace_depth = config_value
        .get("max_trace_depth")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    // Parse excluded paths
    let exclude_paths = config_value
        .get("exclude_paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_else(|| PerformanceAnalyzerConfig::default().exclude_paths);

    // Parse trace headers flag
    let trace_headers = config_value
        .get("trace_headers")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| PerformanceAnalyzerConfig::default().trace_headers);

    // Parse storage configuration
    let storage = if let Some(storage_val) = config_value.get("storage") {
         match serde_json::from_value::<MetricsStorage>(storage_val.clone()) {
             Ok(s) => s,
             Err(e) => {
                 warn!("Failed to parse 'storage' config: {}. Using default Memory storage.", e);
                 PerformanceAnalyzerConfig::default().storage
             }
         }
    } else {
        PerformanceAnalyzerConfig::default().storage
    };

    Ok(PerformanceAnalyzerConfig {
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
    })
}

impl PerformanceAnalyzer {
    // Finalize and store metrics for a completed request
    async fn finalize_metrics(&self, session: &Session, ctx: &mut RouterContext, config: &PerformanceAnalyzerConfig) -> Result<()> {
         const CONTEXT_KEY: &str = "performance_analyzer_context";
         let perf_ctx_val = ctx.plugins_data.get(CONTEXT_KEY);

         if perf_ctx_val.is_none() {
             debug!("No performance context found for request ID: {}, skipping metrics finalization.", ctx.request_id);
             return Ok(()); // Not sampled or context missing
         }

         let mut perf_ctx: PerformancePluginContext = serde_json::from_value(perf_ctx_val.unwrap().clone())?;

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
         let client_ip = session.client_ip().map(|ip| ip.to_string());

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
     <p>Displaying metrics based on configured storage (currently Memory - max {} entries). Sampling rate: {}.</p>
     {}
     {}
     <p><a href="{}/api/metrics">View Raw Metrics (JSON)</a></p>
     <p><a href="{}/api/summary">View Summary (JSON)</a></p>
     <p><a href="{}/api/hotspots">View Hotspots (JSON)</a></p>
 </body>
 </html>
         "#,
         config.storage_to_string(), // Helper needed
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
    async fn handle_api_request(&self, session: &mut Session, path_suffix: &str, config: &PerformanceAnalyzerConfig) -> Result<ResponseHeader> {
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

    // Helper to get string representation of storage config
    fn storage_to_string(&self) -> String {
        // This needs access to the config, which isn't directly available here.
        // We'll pass config into serve_ui instead.
         // Placeholder - implement properly in serve_ui
        "Memory".to_string()
    }

}

#[async_trait]
impl Plugin for PerformanceAnalyzer {
    fn name(&self) -> Cow<'static, str> {
        "PerformanceAnalyzer".into()
    }

    fn new(plugin_config: Option<&RoutePlugin>) -> Result<Self>
    where
        Self: Sized,
    {
         info!("Creating new PerformanceAnalyzer plugin instance.");
         let config = match parse_plugin_config(plugin_config) {
             Ok(cfg) => cfg,
             Err(e) => {
                 error!("Failed to parse PerformanceAnalyzer config: {}. Using default.", e);
                 PerformanceAnalyzerConfig::default()
             }
         };
          info!("PerformanceAnalyzer config loaded: {:?}", config);

         Ok(Self {
             config: Arc::new(Mutex::new(config)),
             metrics: Arc::new(Mutex::new(Vec::new())), // Initialize in-memory storage
             hot_spots: Arc::new(Mutex::new(Vec::new())),
         })
    }

    async fn start(&self) -> Result<(), PluginError> {
        info!("PerformanceAnalyzer plugin started.");
        // TODO: Initialize external storage backends here based on config
        Ok(())
    }

    async fn stop(&self) -> Result<(), PluginError> {
        info!("PerformanceAnalyzer plugin stopped.");
         // TODO: Cleanup external storage resources here
        Ok(())
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        const CONTEXT_KEY: &str = "performance_analyzer_context";
        let config_guard = self.config.lock().await;

        if !config_guard.enabled {
            return Ok((false, None));
        }

        let req_path = session.req_header().uri.path();

        // Check excluded paths
        if config_guard.exclude_paths.iter().any(|p| req_path.starts_with(p)) {
             debug!("Path {} excluded from performance analysis.", req_path);
             return Ok((false, None));
        }

        match step {
            PluginStep::Request => {
                 // Handle UI and API requests first
                 if req_path == config_guard.ui_endpoint {
                    info!("Serving PerformanceAnalyzer UI for path: {}", req_path);
                    return match self.serve_ui(session, ctx, &config_guard).await {
                         Ok(response_header) => Ok((true, Some(HttpResponse::new(response_header, None)))),
                         Err(e) => {
                            error!("Error serving PerformanceAnalyzer UI: {}", e);
                            let err_resp = ResponseHeader::build(StatusCode::INTERNAL_SERVER_ERROR, None)?;
                            session.write_response_header(Box::new(err_resp.clone()), true).await?;
                            Ok((true, Some(HttpResponse::new(err_resp, None))))
                         }
                     };
                 } else if req_path.starts_with(&format!("{}/api/", config_guard.ui_endpoint)) {
                    let path_suffix = req_path.strip_prefix(&config_guard.ui_endpoint).unwrap_or(req_path);
                    info!("Handling PerformanceAnalyzer API request for path suffix: {}", path_suffix);
                    return match self.handle_api_request(session, path_suffix, &config_guard).await {
                         Ok(response_header) => Ok((true, Some(HttpResponse::new(response_header, None)))),
                         Err(e) => {
                             error!("Error handling PerformanceAnalyzer API request: {}", e);
                             let err_resp = ResponseHeader::build(StatusCode::INTERNAL_SERVER_ERROR, None)?;
                             let body = format!(r#"{{"error": "API processing error: {}"}}"#, e).into_bytes();
                             session.write_response_header(Box::new(err_resp.clone()), false).await?;
                             session.write_response_body(Some(bytes::Bytes::from(body)), true).await?;
                             Ok((true, Some(HttpResponse::new(err_resp, None))))
                         }
                     };
                 }

                // Apply sampling
                let is_sampled = rand::thread_rng().gen_range(0.0..1.0) < config_guard.sample_rate;
                debug!("Request ID {} sampling decision: {}", ctx.request_id, is_sampled);

                // Initialize context data
                let perf_ctx = PerformancePluginContext::new(is_sampled);
                ctx.plugins_data.insert(CONTEXT_KEY.to_string(), serde_json::to_value(perf_ctx)?);

                Ok((false, None))
            }
            PluginStep::ProxyUpstream => {
                 if let Some(perf_ctx_val) = ctx.plugins_data.get_mut(CONTEXT_KEY) {
                     match serde_json::from_value::<PerformancePluginContext>(perf_ctx_val.clone()) {
                        Ok(mut perf_ctx) => {
                             if perf_ctx.is_sampled {
                                perf_ctx.upstream_request_start_unix_ns = Some(PerformancePluginContext::now_ns());
                                *perf_ctx_val = serde_json::to_value(perf_ctx)?;
                                debug!("Marked upstream_request_start for request ID: {}", ctx.request_id);
                             }
                         }
                         Err(e) => error!("Failed to deserialize performance context in ProxyUpstream step: {}", e),
                     }
                 }
                Ok((false, None))
            }
             _ => Ok((false, None)), // Ignore other steps in handle_request
        }
    }

    async fn handle_response(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
        _upstream_response: &mut ResponseHeader, // We use ctx.upstream_response in Log step
    ) -> Result<bool> {
        const CONTEXT_KEY: &str = "performance_analyzer_context";
        let config_guard = self.config.lock().await;

         if !config_guard.enabled {
            return Ok(false);
        }

        match step {
            PluginStep::Response => {
                // Mark upstream response received time
                 if let Some(perf_ctx_val) = ctx.plugins_data.get_mut(CONTEXT_KEY) {
                     match serde_json::from_value::<PerformancePluginContext>(perf_ctx_val.clone()) {
                         Ok(mut perf_ctx) => {
                             if perf_ctx.is_sampled {
                                 perf_ctx.upstream_response_received_unix_ns = Some(PerformancePluginContext::now_ns());
                                 // Mark request processing end approx here (end of filters before upstream)
                                 perf_ctx.request_processing_end_unix_ns = perf_ctx.upstream_request_start_unix_ns;
                                 *perf_ctx_val = serde_json::to_value(perf_ctx)?;
                                 debug!("Marked upstream_response_received for request ID: {}", ctx.request_id);
                             }
                         }
                         Err(e) => error!("Failed to deserialize performance context in Response step: {}", e),
                     }
                 }
                Ok(false) // Don't modify headers here
            }
            PluginStep::Log => {
                // Finalize and store metrics
                 match self.finalize_metrics(session, ctx, &config_guard).await {
                     Ok(_) => debug!("Successfully finalized metrics for request ID: {}", ctx.request_id),
                     Err(e) => error!("Error finalizing metrics for request ID {}: {}", ctx.request_id, e),
                 }
                Ok(false)
            }
             _ => Ok(false), // Ignore other steps
        }
    }
}

// Remove old test module
// #[cfg(test)]
// mod tests {
// ... tests ...
// } 