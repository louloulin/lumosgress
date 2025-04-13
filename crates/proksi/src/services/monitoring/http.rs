use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::proxy::Session;
use serde::{Deserialize, Serialize};

use super::{AlertSeverity, MonitoringService};

/// Request to acknowledge an alert
#[derive(Debug, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub alert_id: String,
}

/// Response for alert acknowledgement
#[derive(Debug, Serialize)]
pub struct AcknowledgeAlertResponse {
    pub success: bool,
    pub message: String,
}

/// Filter for alert queries
#[derive(Debug, Deserialize)]
pub struct AlertsQueryParams {
    pub severity: Option<String>,
    pub acknowledged: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// Filter for metrics queries
#[derive(Debug, Deserialize)]
pub struct MetricsQueryParams {
    pub metric_name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub tags: Option<HashMap<String, String>>,
}

/// Monitoring HTTP handler
pub struct MonitoringHttpHandler {
    service: Arc<MonitoringService>,
}

impl MonitoringHttpHandler {
    pub fn new(service: Arc<MonitoringService>) -> Self {
        Self { service }
    }
    
    /// Handle monitoring HTTP requests
    pub async fn handle_request(&self, session: &mut Session, request: &RequestHeader) -> bool {
        let path = request.uri().path();
        
        match path {
            // Dashboard UI
            "/monitoring" | "/monitoring/" => {
                self.serve_dashboard(session).await
            },
            
            // API endpoints
            "/monitoring/api/alerts" => {
                match request.method().as_str() {
                    "GET" => self.handle_get_alerts(session, request).await,
                    "POST" => self.handle_acknowledge_alert(session, request).await,
                    _ => self.handle_method_not_allowed(session).await,
                }
            },
            
            "/monitoring/api/metrics" => {
                match request.method().as_str() {
                    "GET" => self.handle_get_metrics(session, request).await,
                    _ => self.handle_method_not_allowed(session).await,
                }
            },
            
            "/monitoring/api/metric-names" => {
                match request.method().as_str() {
                    "GET" => self.handle_get_metric_names(session).await,
                    _ => self.handle_method_not_allowed(session).await,
                }
            },
            
            _ => false, // Not handled by this handler
        }
    }
    
    /// Serve the monitoring dashboard UI
    async fn serve_dashboard(&self, session: &mut Session) -> bool {
        let html = self.render_dashboard();
        
        let mut response = ResponseHeader::build_200();
        response.append_header("Content-Type", "text/html; charset=utf-8")?;
        response.append_header("Content-Length", html.len().to_string())?;
        
        session.write_response_header(Box::new(response), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(html.into_bytes())), true).await?;
        
        Ok(true)
    }
    
    /// Handle GET request for alerts
    async fn handle_get_alerts(&self, session: &mut Session, request: &RequestHeader) -> bool {
        // Parse query parameters
        let query = request.uri().query().unwrap_or("");
        let query_params: HashMap<String, String> = parse_query_string(query);
        
        // Extract parameters
        let severity = query_params.get("severity").and_then(|s| match s.as_str() {
            "info" => Some(AlertSeverity::Info),
            "warning" => Some(AlertSeverity::Warning),
            "critical" => Some(AlertSeverity::Critical),
            _ => None,
        });
        
        let acknowledged = query_params.get("acknowledged").map(|s| s == "true");
        
        let start_time = query_params.get("start_time")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
            
        let end_time = query_params.get("end_time")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        // Get alerts from the service
        let alerts = self.service.get_alerts(severity, acknowledged, start_time, end_time).await;
        
        // Serialize to JSON
        let json = serde_json::to_string(&alerts).unwrap_or_else(|_| "[]".to_string());
        
        // Send response
        let mut response = ResponseHeader::build_200();
        response.append_header("Content-Type", "application/json")?;
        response.append_header("Content-Length", json.len().to_string())?;
        
        session.write_response_header(Box::new(response), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(json.into_bytes())), true).await?;
        
        Ok(true)
    }
    
    /// Handle POST request to acknowledge an alert
    async fn handle_acknowledge_alert(&self, session: &mut Session, request: &RequestHeader) -> bool {
        // Read request body
        let mut body = Vec::new();
        while let Some(chunk) = session.read_request_body().await? {
            body.extend_from_slice(&chunk);
        }
        
        // Parse JSON
        let acknowledge_request: AcknowledgeAlertRequest = match serde_json::from_slice(&body) {
            Ok(req) => req,
            Err(_) => {
                return self.handle_bad_request(session, "Invalid JSON payload").await;
            }
        };
        
        // Acknowledge the alert
        let success = self.service.acknowledge_alert(&acknowledge_request.alert_id).await;
        
        // Prepare response
        let response = AcknowledgeAlertResponse {
            success,
            message: if success {
                "Alert acknowledged successfully".to_string()
            } else {
                "Alert not found".to_string()
            },
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
        
        // Send response
        let mut response_header = ResponseHeader::build_200();
        response_header.append_header("Content-Type", "application/json")?;
        response_header.append_header("Content-Length", json.len().to_string())?;
        
        session.write_response_header(Box::new(response_header), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(json.into_bytes())), true).await?;
        
        Ok(true)
    }
    
    /// Handle GET request for metrics
    async fn handle_get_metrics(&self, session: &mut Session, request: &RequestHeader) -> bool {
        // Parse query parameters
        let query = request.uri().query().unwrap_or("");
        let query_params: HashMap<String, String> = parse_query_string(query);
        
        // Extract parameters
        let metric_name = match query_params.get("name") {
            Some(name) => name,
            None => {
                return self.handle_bad_request(session, "Missing required parameter: name").await;
            }
        };
        
        let start_time = query_params.get("start_time")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
            
        let end_time = query_params.get("end_time")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        // Extract tags (format: tag1=value1,tag2=value2)
        let tags = query_params.get("tags").map(|tags_str| {
            let mut map = HashMap::new();
            for tag_pair in tags_str.split(',') {
                if let Some((key, value)) = tag_pair.split_once('=') {
                    map.insert(key.to_string(), value.to_string());
                }
            }
            map
        });
        
        // Get metrics from the service
        let metrics = self.service.get_metrics(metric_name, start_time, end_time, tags).await;
        
        // Serialize to JSON
        let json = serde_json::to_string(&metrics).unwrap_or_else(|_| "[]".to_string());
        
        // Send response
        let mut response = ResponseHeader::build_200();
        response.append_header("Content-Type", "application/json")?;
        response.append_header("Content-Length", json.len().to_string())?;
        
        session.write_response_header(Box::new(response), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(json.into_bytes())), true).await?;
        
        Ok(true)
    }
    
    /// Handle GET request for metric names
    async fn handle_get_metric_names(&self, session: &mut Session, _request: &RequestHeader) -> bool {
        // Get metric names from the service
        let metric_names = self.service.get_metric_names();
        
        // Serialize to JSON
        let json = serde_json::to_string(&metric_names).unwrap_or_else(|_| "[]".to_string());
        
        // Send response
        let mut response = ResponseHeader::build_200();
        response.append_header("Content-Type", "application/json")?;
        response.append_header("Content-Length", json.len().to_string())?;
        
        session.write_response_header(Box::new(response), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(json.into_bytes())), true).await?;
        
        Ok(true)
    }
    
    /// Handle method not allowed
    async fn handle_method_not_allowed(&self, session: &mut Session) -> bool {
        let mut response = ResponseHeader::build(405, Some("Method Not Allowed"))?;
        response.append_header("Content-Length", "0")?;
        
        session.write_response_header(Box::new(response), false).await?;
        session.write_response_body(None, true).await?;
        
        Ok(true)
    }
    
    /// Handle bad request
    async fn handle_bad_request(&self, session: &mut Session, message: &str) -> bool {
        let json = format!("{{\"error\": \"{}\"}}", message);
        
        let mut response = ResponseHeader::build(400, Some("Bad Request"))?;
        response.append_header("Content-Type", "application/json")?;
        response.append_header("Content-Length", json.len().to_string())?;
        
        session.write_response_header(Box::new(response), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(json.into_bytes())), true).await?;
        
        Ok(true)
    }
    
    /// Render the monitoring dashboard HTML
    fn render_dashboard(&self) -> String {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Proksi Monitoring Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; line-height: 1.6; }
        .container { max-width: 1200px; margin: 0 auto; }
        header { background: #f4f4f4; padding: 20px; margin-bottom: 20px; border-radius: 5px; }
        h1, h2, h3 { color: #333; }
        .metrics-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(400px, 1fr)); gap: 20px; }
        .metric-card { background: white; border: 1px solid #ddd; border-radius: 5px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .metric-value { font-size: 24px; font-weight: bold; color: #2c3e50; margin: 15px 0; }
        .chart-container { height: 250px; margin-top: 20px; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { text-align: left; padding: 12px; border-bottom: 1px solid #ddd; }
        th { background-color: #f4f4f4; }
        tr:hover { background-color: #f9f9f9; }
        .alert { background-color: #f8d7da; color: #721c24; padding: 10px; border-radius: 5px; margin-bottom: 10px; }
        .alert-warning { background-color: #fff3cd; color: #856404; }
        .alert-info { background-color: #d1ecf1; color: #0c5460; }
        .tabs { display: flex; margin-bottom: 20px; border-bottom: 1px solid #ddd; }
        .tab { padding: 10px 20px; cursor: pointer; }
        .tab.active { border-bottom: 2px solid #0066cc; font-weight: bold; }
        .tab-content { display: none; }
        .tab-content.active { display: block; }
        .flex-between { display: flex; justify-content: space-between; align-items: center; }
        .button { background: #0066cc; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; }
        .button:hover { background: #0055aa; }
    </style>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <div class="container">
        <header>
            <h1>Proksi Monitoring Dashboard</h1>
            <p>Real-time monitoring and alerting for your API Gateway</p>
        </header>
        
        <div class="tabs">
            <div class="tab active" data-tab="overview">Overview</div>
            <div class="tab" data-tab="alerts">Alerts</div>
            <div class="tab" data-tab="metrics">Metrics</div>
            <div class="tab" data-tab="settings">Settings</div>
        </div>
        
        <div id="overview" class="tab-content active">
            <div class="flex-between">
                <h2>System Overview</h2>
                <div>
                    <select id="time-range">
                        <option value="1h">Last Hour</option>
                        <option value="24h" selected>Last 24 Hours</option>
                        <option value="7d">Last 7 Days</option>
                        <option value="30d">Last 30 Days</option>
                    </select>
                    <button class="button" id="refresh-button">Refresh</button>
                </div>
            </div>
            
            <div class="metrics-grid">
                <div class="metric-card">
                    <h3>Request Count</h3>
                    <div class="metric-value" id="request-count">-</div>
                    <div class="chart-container">
                        <canvas id="request-chart"></canvas>
                    </div>
                </div>
                <div class="metric-card">
                    <h3>Response Time (ms)</h3>
                    <div class="metric-value" id="response-time">-</div>
                    <div class="chart-container">
                        <canvas id="latency-chart"></canvas>
                    </div>
                </div>
                <div class="metric-card">
                    <h3>Error Rate</h3>
                    <div class="metric-value" id="error-rate">-</div>
                    <div class="chart-container">
                        <canvas id="error-chart"></canvas>
                    </div>
                </div>
                <div class="metric-card">
                    <h3>Active Alerts</h3>
                    <div class="metric-value" id="alert-count">-</div>
                    <div id="alert-summary">
                        <div class="alert-info">No active alerts</div>
                    </div>
                </div>
            </div>
        </div>
        
        <div id="alerts" class="tab-content">
            <div class="flex-between">
                <h2>Alert Management</h2>
                <div>
                    <select id="alert-filter">
                        <option value="all">All Alerts</option>
                        <option value="active">Active</option>
                        <option value="acknowledged">Acknowledged</option>
                    </select>
                </div>
            </div>
            
            <table id="alerts-table">
                <thead>
                    <tr>
                        <th>Time</th>
                        <th>Metric</th>
                        <th>Value</th>
                        <th>Threshold</th>
                        <th>Severity</th>
                        <th>Status</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td colspan="7" class="text-center">Loading alerts...</td>
                    </tr>
                </tbody>
            </table>
        </div>
        
        <div id="metrics" class="tab-content">
            <div class="flex-between">
                <h2>Metric Explorer</h2>
                <div>
                    <select id="metric-selector">
                        <option value="">Select a metric</option>
                    </select>
                </div>
            </div>
            
            <div class="chart-container" style="height: 400px">
                <canvas id="metric-explorer-chart"></canvas>
            </div>
            
            <h3>Metric Details</h3>
            <table id="metric-details">
                <thead>
                    <tr>
                        <th>Time</th>
                        <th>Value</th>
                        <th>Tags</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td colspan="3" class="text-center">Select a metric to view details</td>
                    </tr>
                </tbody>
            </table>
        </div>
        
        <div id="settings" class="tab-content">
            <h2>Monitoring Settings</h2>
            
            <div class="metric-card">
                <h3>Alert Thresholds</h3>
                <table id="threshold-settings">
                    <thead>
                        <tr>
                            <th>Metric</th>
                            <th>Description</th>
                            <th>Current Threshold</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td colspan="4" class="text-center">Loading settings...</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            
            <div class="metric-card">
                <h3>Alert Notifications</h3>
                <form id="notification-settings">
                    <div>
                        <label>
                            <input type="checkbox" name="enable_email"> Email Alerts
                        </label>
                        <input type="email" name="email" placeholder="Email address">
                    </div>
                    <div>
                        <label>
                            <input type="checkbox" name="enable_slack"> Slack Alerts
                        </label>
                        <input type="text" name="slack_webhook" placeholder="Slack webhook URL">
                    </div>
                    <div>
                        <label>
                            <input type="checkbox" name="enable_webhook"> Webhook Alerts
                        </label>
                        <input type="text" name="webhook_url" placeholder="Webhook URL">
                    </div>
                    <button type="submit" class="button">Save Settings</button>
                </form>
            </div>
        </div>
    </div>
    
    <script>
        // Tab switching logic
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
                
                tab.classList.add('active');
                document.getElementById(tab.dataset.tab).classList.add('active');
            });
        });
        
        // Fetch metric names
        fetch('/monitoring/api/metric-names')
            .then(response => response.json())
            .then(names => {
                const selector = document.getElementById('metric-selector');
                names.forEach(name => {
                    const option = document.createElement('option');
                    option.value = name;
                    option.textContent = name;
                    selector.appendChild(option);
                });
            });
        
        // Fetch alerts
        function fetchAlerts() {
            fetch('/monitoring/api/alerts')
                .then(response => response.json())
                .then(alerts => {
                    const tbody = document.querySelector('#alerts-table tbody');
                    tbody.innerHTML = '';
                    
                    if (alerts.length === 0) {
                        const row = document.createElement('tr');
                        row.innerHTML = '<td colspan="7" class="text-center">No alerts found</td>';
                        tbody.appendChild(row);
                        return;
                    }
                    
                    alerts.forEach(alert => {
                        const row = document.createElement('tr');
                        const date = new Date(alert.timestamp);
                        
                        row.innerHTML = `
                            <td>${date.toLocaleString()}</td>
                            <td>${alert.metric_name}</td>
                            <td>${alert.value.toFixed(2)}</td>
                            <td>${alert.threshold.toFixed(2)}</td>
                            <td>${alert.severity}</td>
                            <td>${alert.acknowledged ? 'Acknowledged' : 'Active'}</td>
                            <td>
                                ${!alert.acknowledged ? 
                                    `<button class="ack-button" data-id="${alert.id}">Acknowledge</button>` : 
                                    ''}
                            </td>
                        `;
                        tbody.appendChild(row);
                    });
                    
                    // Add event listeners to acknowledge buttons
                    document.querySelectorAll('.ack-button').forEach(button => {
                        button.addEventListener('click', () => {
                            const alertId = button.dataset.id;
                            fetch('/monitoring/api/alerts', {
                                method: 'POST',
                                headers: {
                                    'Content-Type': 'application/json'
                                },
                                body: JSON.stringify({ alert_id: alertId })
                            })
                            .then(response => response.json())
                            .then(result => {
                                if (result.success) {
                                    fetchAlerts(); // Refresh the table
                                }
                            });
                        });
                    });
                });
        }
        
        // Initialize the page
        document.addEventListener('DOMContentLoaded', () => {
            fetchAlerts();
            
            // Set up refresh button
            document.getElementById('refresh-button').addEventListener('click', () => {
                fetchAlerts();
                // Other refresh logic here
            });
            
            // Set up metric selector
            document.getElementById('metric-selector').addEventListener('change', (e) => {
                const metricName = e.target.value;
                if (!metricName) return;
                
                fetch(`/monitoring/api/metrics?name=${metricName}`)
                    .then(response => response.json())
                    .then(metrics => {
                        // Update chart and details table
                        // This would be implemented with actual chart.js code
                        console.log('Fetched metrics:', metrics);
                    });
            });
        });
    </script>
</body>
</html>"#;
        
        html.to_string()
    }
}

/// Parse a query string into a HashMap
fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            map.insert(
                key.to_string(), 
                urlencoding::decode(value).unwrap_or_else(|_| value.into()).into_owned(),
            );
        }
    }
    
    map
} 