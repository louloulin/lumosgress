use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use pingora::server::{ListenFds, ShutdownWatch};
use pingora::services::Service;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::time::interval;
use tracing::{info, warn};

use crate::config::{AlertChannelType, Config};

/// The monitoring service responsible for collecting metrics and sending alerts
pub struct MonitoringService {
    config: Arc<Config>,
    metrics: Arc<DashMap<String, Vec<MetricPoint>>>,
    alerts: Arc<Mutex<Vec<Alert>>>,
    last_cleanup: Arc<Mutex<DateTime<Utc>>>,
}

/// A single metric point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Timestamp when the metric was collected
    pub timestamp: DateTime<Utc>,
    /// Name of the metric
    pub name: String,
    /// Value of the metric
    pub value: f64,
    /// Additional metadata/dimensions
    pub tags: HashMap<String, String>,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// An alert generated from monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique ID of the alert
    pub id: String,
    /// Timestamp when the alert was generated
    pub timestamp: DateTime<Utc>,
    /// Name of the metric that triggered the alert
    pub metric_name: String,
    /// Actual value that triggered the alert
    pub value: f64,
    /// Threshold that was exceeded
    pub threshold: f64,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Human-readable description of the alert
    pub description: String,
    /// Whether the alert is acknowledged
    pub acknowledged: bool,
    /// Additional metadata/dimensions
    pub tags: HashMap<String, String>,
}

impl MonitoringService {
    /// Create a new monitoring service with the given configuration
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            metrics: Arc::new(DashMap::new()),
            alerts: Arc::new(Mutex::new(Vec::new())),
            last_cleanup: Arc::new(Mutex::new(Utc::now())),
        }
    }

    /// Add a metric point to the collection
    pub async fn add_metric(&self, metric: MetricPoint) {
        if !self.config.monitoring.enabled {
            return;
        }

        // Skip sampling if needed
        if rand::random::<f64>() > self.config.monitoring.sample_rate {
            return;
        }

        // Store the metric
        self.metrics
            .entry(metric.name.clone())
            .or_insert_with(Vec::new)
            .push(metric.clone());

        // Check against thresholds and generate alerts if needed
        self.check_thresholds(&metric).await;
    }

    /// Check a metric against configured thresholds
    async fn check_thresholds(&self, metric: &MetricPoint) {
        // Find the metric config
        let metric_config = self.config.monitoring.metrics.iter().find(|m| m.name == metric.name);
        
        if let Some(config) = metric_config {
            // Check specific metric threshold
            if let Some(threshold) = config.threshold {
                if metric.value > threshold {
                    let alert = Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        metric_name: metric.name.clone(),
                        value: metric.value,
                        threshold,
                        severity: if metric.value > threshold * 1.5 {
                            AlertSeverity::Critical
                        } else if metric.value > threshold * 1.2 {
                            AlertSeverity::Warning
                        } else {
                            AlertSeverity::Info
                        },
                        description: format!(
                            "Metric {} exceeded threshold: {} > {}",
                            metric.name, metric.value, threshold
                        ),
                        acknowledged: false,
                        tags: metric.tags.clone(),
                    };
                    
                    // Send the alert
                    self.send_alert(alert.clone()).await;
                    
                    // Store the alert
                    let mut alerts = self.alerts.lock().await;
                    alerts.push(alert);
                }
            }
        }
    }

    /// Send an alert through configured channels
    async fn send_alert(&self, alert: Alert) {
        for channel in &self.config.monitoring.alert_channels {
            match channel.channel_type {
                AlertChannelType::Log => {
                    match alert.severity {
                        AlertSeverity::Info => info!("{}", alert.description),
                        AlertSeverity::Warning => warn!("{}", alert.description),
                        AlertSeverity::Critical => warn!("CRITICAL: {}", alert.description),
                    }
                }
                AlertChannelType::Webhook => {
                    if let Some(url) = channel.config.get("url") {
                        info!("Sending alert to webhook {}: {}", url, alert.description);
                        // In a real implementation, make an HTTP request to the webhook
                    }
                }
                AlertChannelType::Email => {
                    if let Some(recipient) = channel.config.get("recipient") {
                        info!("Sending alert to email {}: {}", recipient, alert.description);
                        // In a real implementation, send an email
                    }
                }
                AlertChannelType::Slack => {
                    if let Some(webhook) = channel.config.get("webhook_url") {
                        info!("Sending alert to Slack webhook {}: {}", webhook, alert.description);
                        // In a real implementation, post to Slack
                    }
                }
            }
        }
    }

    /// Clean up old metrics data based on retention period
    #[allow(dead_code)]
    async fn cleanup_old_data(&self) {
        let retention_days = self.config.monitoring.retention_days;
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        
        // Update last cleanup time
        let mut last_cleanup = self.last_cleanup.lock().await;
        *last_cleanup = Utc::now();
        
        // Clean up metrics
        for mut metrics in self.metrics.iter_mut() {
            metrics.retain(|m| m.timestamp >= cutoff);
        }
        
        // Clean up alerts
        let mut alerts = self.alerts.lock().await;
        alerts.retain(|a| a.timestamp >= cutoff);
        
        info!("Cleaned up monitoring data older than {} days", retention_days);
    }

    /// Get alerts matching filters
    pub async fn get_alerts(&self, 
        severity: Option<AlertSeverity>,
        acknowledged: Option<bool>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Vec<Alert> {
        let alerts = self.alerts.lock().await;
        
        alerts.iter()
            .filter(|a| {
                // Filter by severity if provided
                if let Some(sev) = &severity {
                    if &a.severity != sev {
                        return false;
                    }
                }
                
                // Filter by acknowledged status if provided
                if let Some(ack) = acknowledged {
                    if a.acknowledged != ack {
                        return false;
                    }
                }
                
                // Filter by start time if provided
                if let Some(start) = start_time {
                    if a.timestamp < start {
                        return false;
                    }
                }
                
                // Filter by end time if provided
                if let Some(end) = end_time {
                    if a.timestamp > end {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect()
    }
    
    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> bool {
        let mut alerts = self.alerts.lock().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            true
        } else {
            false
        }
    }
    
    /// Get metric data
    pub async fn get_metrics(
        &self,
        metric_name: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Vec<MetricPoint> {
        if let Some(metrics) = self.metrics.get(metric_name) {
            metrics.iter()
                .filter(|m| {
                    // Filter by start time if provided
                    if let Some(start) = start_time {
                        if m.timestamp < start {
                            return false;
                        }
                    }
                    
                    // Filter by end time if provided
                    if let Some(end) = end_time {
                        if m.timestamp > end {
                            return false;
                        }
                    }
                    
                    // Filter by tags if provided
                    if let Some(tag_filters) = &tags {
                        for (key, value) in tag_filters {
                            if m.tags.get(key) != Some(value) {
                                return false;
                            }
                        }
                    }
                    
                    true
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get available metric names
    pub fn get_metric_names(&self) -> Vec<String> {
        self.metrics.iter().map(|m| m.key().clone()).collect()
    }
}

#[async_trait]
impl Service for MonitoringService {
    async fn start_service(&mut self, _fds: Option<ListenFds>, _shutdown: ShutdownWatch) {
        if !self.config.monitoring.enabled {
            info!("Monitoring service is disabled");
            return;
        }

        info!("Starting monitoring service");
        
        // Set up periodic data cleanup
        let metrics = self.metrics.clone();
        let alerts = self.alerts.clone();
        let last_cleanup = self.last_cleanup.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Hourly cleanup
            
            loop {
                interval.tick().await;
                
                let retention_days = config.monitoring.retention_days;
                let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
                
                // Update last cleanup time
                let mut last_cleanup_lock = last_cleanup.lock().await;
                *last_cleanup_lock = Utc::now();
                
                // Clean up metrics
                for mut metrics_entry in metrics.iter_mut() {
                    metrics_entry.retain(|m| m.timestamp >= cutoff);
                }
                
                // Clean up alerts
                let mut alerts_lock = alerts.lock().await;
                alerts_lock.retain(|a| a.timestamp >= cutoff);
                
                info!("Cleaned up monitoring data older than {} days", retention_days);
            }
        });
    }

    fn name(&self) -> &'static str {
        "monitoring_service"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_config() -> Arc<Config> {
        let mut config = Config::default();
        config.monitoring.enabled = true;
        config.monitoring.alert_threshold = 0.9;
        config.monitoring.retention_days = 7;
        
        // Add a test metric config
        config.monitoring.metrics = vec![
            crate::config::MetricConfig {
                name: "test_metric".to_string(),
                description: "Test metric".to_string(),
                enabled: true,
                threshold: Some(10.0),
            },
        ];
        
        Arc::new(config)
    }

    #[tokio::test]
    async fn test_add_metric() {
        let config = create_test_config();
        let service = MonitoringService::new(config);
        
        let metric = MetricPoint {
            timestamp: Utc::now(),
            name: "test_metric".to_string(),
            value: 5.0,
            tags: HashMap::new(),
        };
        
        service.add_metric(metric.clone()).await;
        
        // Verify the metric was added
        let metrics = service.get_metrics("test_metric", None, None, None).await;
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "test_metric");
        assert_eq!(metrics[0].value, 5.0);
    }
    
    #[tokio::test]
    async fn test_alert_generation() {
        let config = create_test_config();
        let service = MonitoringService::new(config);
        
        // Add a metric that exceeds the threshold
        let metric = MetricPoint {
            timestamp: Utc::now(),
            name: "test_metric".to_string(),
            value: 15.0, // > 10.0 threshold
            tags: HashMap::new(),
        };
        
        service.add_metric(metric).await;
        
        // Verify an alert was generated
        let alerts = service.get_alerts(None, None, None, None).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].metric_name, "test_metric");
        assert_eq!(alerts[0].value, 15.0);
        assert_eq!(alerts[0].threshold, 10.0);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning); // 15 > 10 * 1.2
    }
} 