use std::sync::Arc;
use anyhow::Result;
use pingora::proxy::Session;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::{info, warn};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::http::StatusCode;
use bytes;

use crate::{
    config::RoutePlugin,
    proxy_server::https_proxy::RouterContext,
};

use crate::plugins::MiddlewarePlugin;

mod anomaly_detection;

pub use anomaly_detection::{
    AnomalyDetectionConfig, AnomalyDetectionType, 
    AnomalyAlert, AnomalySeverity, AlertChannelConfig, AlertChannelType
};
use anomaly_detection::AnomalyDetector;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnalyticsStorageType {
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

pub struct AiAnalytics {
    config: Arc<HashMap<String, AiAnalyticsConfig>>,
    storage: HashMap<String, Arc<Mutex<dyn AnalyticsStorage>>>,
    metrics: Arc<Mutex<HashMap<String, Vec<AnalyticsMetric>>>>,
    anomaly_detectors: HashMap<String, Arc<Mutex<AnomalyDetector>>>,
    last_prune_time: DateTime<Utc>,
}

#[async_trait]
trait AnalyticsStorage: Send + Sync {
    async fn store_metric(&mut self, metric: AnalyticsMetric) -> Result<()>;
    async fn get_metrics(&self, metric_name: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<AnalyticsMetric>>;
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
            config: Arc::new(HashMap::new()),
            storage: HashMap::new(),
            metrics: Arc::new(Mutex::new(HashMap::new())),
            anomaly_detectors: HashMap::new(),
            last_prune_time: Utc::now(),
        }
    }

    fn get_config(&self, config_name: &str) -> Option<&AiAnalyticsConfig> {
        self.config.get(config_name)
    }

    async fn initialize_storage(&mut self, config_name: &str, config: &AiAnalyticsConfig) -> Result<()> {
        let storage: Arc<Mutex<dyn AnalyticsStorage>> = match config.storage_type {
            AnalyticsStorageType::InMemory => Arc::new(Mutex::new(InMemoryStorage::new())),
            AnalyticsStorageType::Redis => Arc::new(Mutex::new(RedisStorage::new(config))),
            AnalyticsStorageType::Postgres => Arc::new(Mutex::new(PostgresStorage::new(config))),
            AnalyticsStorageType::Custom(_) => Arc::new(Mutex::new(CustomStorage::new(config))),
        };

        self.storage.insert(config_name.to_string(), storage);
        
        // Initialize anomaly detector if configured
        if let Some(anomaly_config) = &config.anomaly_detection {
            let detector = AnomalyDetector::new(anomaly_config.clone());
            self.anomaly_detectors.insert(config_name.to_string(), Arc::new(Mutex::new(detector)));
        }
        
        Ok(())
    }

    async fn process_request(&self, session: &mut Session, config: &AiAnalyticsConfig) -> Result<()> {
        if !config.metrics_enabled {
            return Ok(());
        }

        // Skip sampling if needed
        if rand::random::<f64>() > config.sampling_rate {
            return Ok(());
        }

        // Collect request metrics
        let metrics = self.collect_request_metrics(session).await?;
        
        // Store metrics if we have any
        if !metrics.is_empty() {
            let mut metrics_map = self.metrics.lock().await;
            for metric in metrics {
                metrics_map.entry(metric.metric_name.clone())
                    .or_insert_with(Vec::new)
                    .push(metric);
            }
        }

        Ok(())
    }

    async fn process_response(&self, session: &mut Session, config: &AiAnalyticsConfig, config_name: &str) -> Result<()> {
        if !config.metrics_enabled {
            return Ok(());
        }

        // Skip sampling if needed
        if rand::random::<f64>() > config.sampling_rate {
            return Ok(());
        }

        // Collect response metrics
        let metrics = self.collect_response_metrics(session).await?;
        
        // Store metrics
        if !metrics.is_empty() {
            if let Some(storage) = self.storage.get(config_name) {
                let mut storage_lock = storage.lock().await;
                for metric in metrics.clone() {
                    storage_lock.store_metric(metric).await?;
                }
            }
            
            // Feed metrics to anomaly detector if enabled
            if let Some(detector) = self.anomaly_detectors.get(config_name) {
                let mut detector_lock = detector.lock().await;
                for metric in &metrics {
                    detector_lock.add_metric(metric.clone());
                }
                
                // Run anomaly detection
                let anomalies = detector_lock.detect_anomalies();
                if !anomalies.is_empty() {
                    info!("Detected {} anomalies in metrics", anomalies.len());
                }
            }
        }

        // Check alert thresholds
        self.check_alerts(config).await?;
        
        // Periodically prune metric history
        if Utc::now() - self.last_prune_time > Duration::hours(1) {
            self.prune_metric_history(config_name).await?;
        }

        Ok(())
    }

    async fn collect_request_metrics(&self, session: &Session) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();
        let timestamp = Utc::now();
        let mut tags = HashMap::new();

        // Add basic request tags
        if let Some(host) = session.req_header().headers.get("host").and_then(|h| h.to_str().ok()) {
            tags.insert("host".to_string(), host.to_string());
        }
        
        let path = session.req_header().uri.path();
        tags.insert("path".to_string(), path.to_string());
        
        let method = session.req_header().method.as_str();
        tags.insert("method".to_string(), method.to_string());

        // Collect request latency - this would need a different approach since request_time isn't available
        // We'll just use a placeholder for now
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "request_latency".to_string(),
            value: 0.0, // Placeholder, actual measurement would need different approach
            tags: tags.clone(),
        });

        // Collect request size - use content length from header or 0 if not available
        let request_size = session.req_header().headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
            
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "request_size".to_string(),
            value: request_size,
            tags: tags.clone(),
        });

        // Collect LLM provider metrics if available
        if let Some(provider) = session.req_header().headers.get("x-llm-provider") {
            let provider_str = provider.to_str().unwrap_or("unknown");
            let mut provider_tags = tags.clone();
            provider_tags.insert("provider".to_string(), provider_str.to_string());
            
            metrics.push(AnalyticsMetric {
                timestamp,
                metric_name: "llm_provider_requests".to_string(),
                value: 1.0,
                tags: provider_tags,
            });
        }

        // Collect model info if available
        if let Some(model) = session.req_header().headers.get("x-llm-model") {
            let model_str = model.to_str().unwrap_or("unknown");
            let mut model_tags = tags.clone();
            model_tags.insert("model".to_string(), model_str.to_string());
            
            metrics.push(AnalyticsMetric {
                timestamp,
                metric_name: "llm_model_requests".to_string(),
                value: 1.0,
                tags: model_tags,
            });
        }

        Ok(metrics)
    }

    async fn collect_response_metrics(&self, session: &Session) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();
        let timestamp = Utc::now();
        let mut tags = HashMap::new();

        // Add basic response tags
        if let Some(host) = session.req_header().headers.get("host").and_then(|h| h.to_str().ok()) {
            tags.insert("host".to_string(), host.to_string());
        }
        
        let path = session.req_header().uri.path();
        tags.insert("path".to_string(), path.to_string());
        
        let method = session.req_header().method.as_str();
        tags.insert("method".to_string(), method.to_string());

        // Since there's no resp_header method, we'd need to access response header differently
        // For now we'll just use placeholder metrics without status-specific logic
        
        // Collect response latency - this would need a different approach
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "response_latency".to_string(),
            value: 0.0, // Placeholder
            tags: tags.clone(),
        });

        // Collect response size - use placeholder for now
        metrics.push(AnalyticsMetric {
            timestamp,
            metric_name: "response_size".to_string(),
            value: 0.0,
            tags: tags.clone(),
        });

        // We can't access status or response headers in this stub implementation
        // In a real implementation, we would need to find the appropriate way
        // to access response headers in pingora's Session structure

        Ok(metrics)
    }

    async fn check_alerts(&self, config: &AiAnalyticsConfig) -> Result<()> {
        let now = Utc::now();
        let start_time = now - chrono::Duration::minutes(5);

        for (metric_name, threshold) in &config.alert_thresholds {
            // Get metrics from the last 5 minutes
            let metrics = self.metrics.lock().await
                .get(metric_name)
                .map(|metrics| {
                    metrics.iter()
                        .filter(|m| m.timestamp >= start_time)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            // Calculate average
            if !metrics.is_empty() {
                let avg = metrics.iter().map(|m| m.value).sum::<f64>() / metrics.len() as f64;
                
                // Check if over threshold
                if avg > *threshold {
                    warn!(
                        "Alert triggered for metric {}: value {} exceeds threshold {}",
                        metric_name, avg, threshold
                    );
                }
            }
        }

        Ok(())
    }
    
    // Prune old metrics to prevent excessive memory usage
    async fn prune_metric_history(&self, config_name: &str) -> Result<()> {
        // Get retention policy from config
        if let Some(config) = self.config.get(config_name) {
            let retention_days = config.retention_days;
            let cutoff = Utc::now() - Duration::days(retention_days as i64);
            
            // Prune in-memory metrics
            let mut metrics = self.metrics.lock().await;
            for values in metrics.values_mut() {
                values.retain(|m| m.timestamp >= cutoff);
            }
            
            // Prune anomaly detector history
            if let Some(detector) = self.anomaly_detectors.get(config_name) {
                let mut detector_lock = detector.lock().await;
                detector_lock.prune_history(Duration::days(retention_days as i64));
            }
        }
        
        Ok(())
    }

    fn render_dashboard(&self, config: &AiAnalyticsConfig) -> String {
        let html = format!(r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Proksi AI Analytics Dashboard</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; line-height: 1.6; }}
                    .container {{ max-width: 1200px; margin: 0 auto; }}
                    header {{ background: #f4f4f4; padding: 20px; margin-bottom: 20px; border-radius: 5px; }}
                    h1, h2, h3 {{ color: #333; }}
                    .metrics-grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(400px, 1fr)); gap: 20px; }}
                    .metric-card {{ background: white; border: 1px solid #ddd; border-radius: 5px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
                    .metric-value {{ font-size: 24px; font-weight: bold; color: #2c3e50; margin: 15px 0; }}
                    .chart-container {{ height: 250px; margin-top: 20px; }}
                    table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
                    th, td {{ text-align: left; padding: 12px; border-bottom: 1px solid #ddd; }}
                    th {{ background-color: #f4f4f4; }}
                    tr:hover {{ background-color: #f9f9f9; }}
                    .alert {{ background-color: #f8d7da; color: #721c24; padding: 10px; border-radius: 5px; margin-bottom: 10px; }}
                    .alert-warning {{ background-color: #fff3cd; color: #856404; }}
                    .alert-info {{ background-color: #d1ecf1; color: #0c5460; }}
                </style>
                <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
            </head>
            <body>
                <div class="container">
                    <header>
                        <h1>Proksi AI Analytics Dashboard</h1>
                        <p>Real-time metrics and analytics for AI gateway traffic</p>
                    </header>
                    
                    <section id="alerts-section">
                        <h2>Active Alerts</h2>
                        <div id="alerts-container">
                            <!-- Alerts will be inserted here -->
                            <p id="no-alerts" class="alert-info">No active alerts at this time.</p>
                        </div>
                    </section>
                    
                    <section>
                        <h2>Overview</h2>
                        <div class="metrics-grid">
                            <div class="metric-card">
                                <h3>Request Count</h3>
                                <div class="metric-value" id="request-count">Loading...</div>
                                <div class="chart-container">
                                    <canvas id="request-chart"></canvas>
                                </div>
                            </div>
                            <div class="metric-card">
                                <h3>Response Time (ms)</h3>
                                <div class="metric-value" id="response-time">Loading...</div>
                                <div class="chart-container">
                                    <canvas id="latency-chart"></canvas>
                                </div>
                            </div>
                            <div class="metric-card">
                                <h3>Error Rate</h3>
                                <div class="metric-value" id="error-rate">Loading...</div>
                                <div class="chart-container">
                                    <canvas id="error-chart"></canvas>
                                </div>
                            </div>
                            <div class="metric-card">
                                <h3>Token Usage</h3>
                                <div class="metric-value" id="token-usage">Loading...</div>
                                <div class="chart-container">
                                    <canvas id="token-chart"></canvas>
                                </div>
                            </div>
                        </div>
                    </section>
                    
                    <section>
                        <h2>LLM Provider Breakdown</h2>
                        <div class="chart-container" style="height: 300px;">
                            <canvas id="provider-chart"></canvas>
                        </div>
                    </section>
                    
                    <section>
                        <h2>Anomaly Detection</h2>
                        <div class="chart-container" style="height: 300px;">
                            <canvas id="anomaly-chart"></canvas>
                        </div>
                        <div id="anomaly-list">
                            <h3>Recent Anomalies</h3>
                            <table id="anomalies-table">
                                <thead>
                                    <tr>
                                        <th>Time</th>
                                        <th>Metric</th>
                                        <th>Value</th>
                                        <th>Expected</th>
                                        <th>Deviation</th>
                                        <th>Severity</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr><td colspan="6">No anomalies detected</td></tr>
                                </tbody>
                            </table>
                        </div>
                    </section>
                    
                    <section>
                        <h2>Recent Requests</h2>
                        <table id="recent-requests">
                            <thead>
                                <tr>
                                    <th>Time</th>
                                    <th>Provider</th>
                                    <th>Model</th>
                                    <th>Latency (ms)</th>
                                    <th>Tokens</th>
                                    <th>Status</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr><td colspan="6">Loading...</td></tr>
                            </tbody>
                        </table>
                    </section>
                </div>
                
                <script>
                    // This would be replaced with actual data fetching and visualization code
                    // For now, we're just showing the dashboard structure
                    
                    // Example data
                    const requestData = {{
                        labels: Array.from({{length: 24}}, (_, idx) => `${{idx}}:00`),
                        values: Array.from({{length: 24}}, () => Math.floor(Math.random() * 100))
                    }};
                    
                    // Initialize charts
                    const requestChart = new Chart(
                        document.getElementById('request-chart'),
                        {{
                            type: 'line',
                            data: {{
                                labels: requestData.labels,
                                datasets: [{{
                                    label: 'Requests',
                                    data: requestData.values,
                                    borderColor: 'rgb(75, 192, 192)',
                                    tension: 0.1
                                }}]
                            }}
                        }}
                    );
                    
                    // Update metrics
                    document.getElementById('request-count').textContent = 
                        requestData.values.reduce((a, b) => a + b, 0);
                    
                    // Example anomaly chart
                    const anomalyData = {{
                        labels: Array.from({{length: 24}}, (_, idx) => `${{idx}}:00`),
                        values: Array.from({{length: 24}}, () => Math.floor(Math.random() * 100)),
                        anomalies: [3, 8, 15].map(idx => ({{
                            x: `${{idx}}:00`,
                            y: Math.floor(Math.random() * 100) + 50
                        }}))
                    }};
                    
                    new Chart(
                        document.getElementById('anomaly-chart'),
                        {{
                            type: 'line',
                            data: {{
                                labels: anomalyData.labels,
                                datasets: [
                                    {{
                                        label: 'Normal Values',
                                        data: anomalyData.values,
                                        borderColor: 'rgb(75, 192, 192)',
                                        tension: 0.1
                                    }},
                                    {{
                                        label: 'Anomalies',
                                        data: anomalyData.anomalies,
                                        backgroundColor: 'rgb(255, 99, 132)',
                                        borderColor: 'rgb(255, 99, 132)',
                                        pointRadius: 6,
                                        pointStyle: 'circle',
                                        showLine: false
                                    }}
                                ]
                            }}
                        }}
                    );
                    
                    // Example alerts
                    const exampleAlerts = [
                        {{ 
                            severity: 'critical',
                            message: 'Response time exceeded threshold by 250%'
                        }},
                        {{ 
                            severity: 'warning',
                            message: 'Error rate increased by 150% in the last 5 minutes'
                        }}
                    ];
                    
                    if (exampleAlerts.length > 0) {{
                        document.getElementById('no-alerts').style.display = 'none';
                        const alertsContainer = document.getElementById('alerts-container');
                        
                        exampleAlerts.forEach(alert => {{
                            const alertDiv = document.createElement('div');
                            alertDiv.className = alert.severity === 'critical' ? 'alert' : 'alert alert-warning';
                            alertDiv.textContent = alert.message;
                            alertsContainer.appendChild(alertDiv);
                        }});
                    }}
                    
                    // Additional charts and metrics would be initialized similarly
                </script>
            </body>
            </html>
        "#);
        
        html
    }

    pub fn handle_dashboard_request(&self, config: &AiAnalyticsConfig) -> Option<String> {
        if config.dashboard_enabled {
            Some(self.render_dashboard(config))
        } else {
            None
        }
    }
}

#[async_trait]
impl MiddlewarePlugin for AiAnalytics {
    async fn request_filter(
        &self,
        session: &mut Session,
        state: &mut RouterContext,
        config: &RoutePlugin,
    ) -> Result<bool> {
        // Get the config name from the plugin config
        let config_name = match &config.config {
            Some(cfg) => match cfg.get("config_name") {
                Some(name) => match name.as_str() {
                    Some(s) => s,
                    None => return Ok(false),
                },
                None => return Ok(false),
            },
            None => return Ok(false),
        };

        // Get the config by name
        if let Some(analytics_config) = self.get_config(config_name) {
            // Check if this is a dashboard request
            if analytics_config.dashboard_enabled {
                if let Some(dashboard_path) = &analytics_config.dashboard_path {
                    let path = session.req_header().uri.path();
                    if path == dashboard_path.as_str() {
                        // This is a dashboard request
                        if let Some(dashboard_html) = self.handle_dashboard_request(analytics_config) {
                            // Respond with the dashboard HTML
                            let content_length = dashboard_html.len();
                            let mut response = ResponseHeader::build(StatusCode::OK, None)?;
                            response.append_header("Content-Type", "text/html; charset=utf-8")?;
                            response.append_header("Content-Length", content_length.to_string())?;
                            
                            session.write_response_header(Box::new(response), false).await?;
                            session.write_response_body(Some(bytes::Bytes::from(dashboard_html)), true).await?;
                            return Ok(true);
                        }
                    }
                }
            }

            // Process the request metrics asynchronously
            match self.process_request(session, analytics_config).await {
                Ok(_) => {},
                Err(e) => warn!("Error processing analytics for request: {:?}", e)
            }
        }

        // Continue processing the request
        Ok(false)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        _upstream_request: &mut RequestHeader,
        _state: &mut RouterContext,
    ) -> Result<()> {
        // No modifications needed for upstream requests
        Ok(())
    }

    async fn response_filter(
        &self,
        session: &mut Session,
        state: &mut RouterContext,
        config: &RoutePlugin,
    ) -> Result<bool> {
        // Get the config name from the plugin config
        let config_name = match &config.config {
            Some(cfg) => match cfg.get("config_name") {
                Some(name) => match name.as_str() {
                    Some(s) => s,
                    None => return Ok(false),
                },
                None => return Ok(false),
            },
            None => return Ok(false),
        };

        // Get the config by name
        if let Some(analytics_config) = self.get_config(config_name) {
            // Process the response metrics asynchronously
            match self.process_response(session, analytics_config, config_name).await {
                Ok(_) => {},
                Err(e) => warn!("Error processing analytics for response: {:?}", e)
            }
        }

        // Continue processing the response
        Ok(false)
    }

    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        _upstream_response: &mut ResponseHeader,
        _state: &mut RouterContext,
    ) -> Result<()> {
        // No modifications needed for upstream responses
        Ok(())
    }
}

// 实现各个存储后端
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

struct RedisStorage {
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
        // TODO: 实现Redis存储逻辑
        // 需要添加redis依赖并实现实际存储
        Ok(())
    }

    async fn get_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>) -> Result<Vec<AnalyticsMetric>> {
        // TODO: 实现Redis查询逻辑
        Ok(vec![])
    }

    async fn get_aggregated_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>, _aggregation: AggregationType) -> Result<Vec<AnalyticsMetric>> {
        // TODO: 实现Redis聚合查询逻辑
        Ok(vec![])
    }
}

struct PostgresStorage {
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
        // TODO: 实现Postgres存储逻辑
        // 需要添加postgres依赖并实现实际存储
        Ok(())
    }

    async fn get_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>) -> Result<Vec<AnalyticsMetric>> {
        // TODO: 实现Postgres查询逻辑
        Ok(vec![])
    }

    async fn get_aggregated_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>, _aggregation: AggregationType) -> Result<Vec<AnalyticsMetric>> {
        // TODO: 实现Postgres聚合查询逻辑
        Ok(vec![])
    }
}

struct CustomStorage {
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
        // TODO: 实现自定义存储逻辑
        Ok(())
    }

    async fn get_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>) -> Result<Vec<AnalyticsMetric>> {
        // TODO: 实现自定义查询逻辑
        Ok(vec![])
    }

    async fn get_aggregated_metrics(&self, _metric_name: &str, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>, _aggregation: AggregationType) -> Result<Vec<AnalyticsMetric>> {
        // TODO: 实现自定义聚合查询逻辑
        Ok(vec![])
    }
} 