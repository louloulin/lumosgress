use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::plugins::ai_analytics::AnalyticsMetric;

/// Types of anomaly detection algorithms
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalyDetectionType {
    /// Detects values outside of Z standard deviations from the mean
    ZScore(f64),
    /// Detects values outside a percentage range of the moving average
    MovingAverage {
        window_size: usize,
        deviation_percent: f64,
    },
    /// Detects sudden changes in the rate of change (first derivative)
    RateOfChange(f64),
}

/// Configuration for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionConfig {
    /// Which algorithm to use for detection
    pub detection_type: AnomalyDetectionType,
    /// Minimum number of data points needed before detection starts
    pub min_data_points: usize,
    /// List of metrics to monitor for anomalies
    pub monitored_metrics: Vec<String>,
    /// How often to run detection (in seconds)
    pub detection_interval_seconds: u64,
    /// Tags to filter metrics by (e.g., only detect anomalies for specific providers)
    pub tag_filters: Option<HashMap<String, String>>,
    /// Whether to suppress repeat alerts within a time window
    pub alert_suppression_window: Option<u64>,
    /// Alert output channel configuration
    pub alert_channel: AlertChannelConfig,
}

/// Configuration for alert delivery channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertChannelConfig {
    /// Type of alert channel
    pub channel_type: AlertChannelType,
    /// Specific configuration for the channel
    pub config: HashMap<String, String>,
}

/// Types of alert channels for sending notifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertChannelType {
    Log,
    Webhook,
    Email,
    Slack,
}

/// Represents an anomaly that was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    /// When the anomaly was detected
    pub timestamp: DateTime<Utc>,
    /// The metric name that triggered the anomaly
    pub metric_name: String,
    /// The actual value that was detected as anomalous
    pub actual_value: f64,
    /// The expected value or range
    pub expected_value: Option<f64>,
    /// How many standard deviations or percentage from normal
    pub deviation: f64,
    /// Tags associated with the anomalous metric 
    pub tags: HashMap<String, String>,
    /// Human-readable description of the anomaly
    pub description: String,
    /// Severity level of the anomaly
    pub severity: AnomalySeverity,
}

/// Severity levels for anomaly alerts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Info,
    Warning,
    Critical,
}

/// Manages anomaly detection for metrics
pub struct AnomalyDetector {
    /// Configuration for the detector
    config: AnomalyDetectionConfig,
    /// Historical metric values for detection
    metric_history: HashMap<String, Vec<AnalyticsMetric>>,
    /// Last time detection was run
    last_detection_time: DateTime<Utc>,
    /// Previous alerts to support suppression
    previous_alerts: Vec<AnomalyAlert>,
}

impl AnomalyDetector {
    /// Create a new anomaly detector with the given configuration
    pub fn new(config: AnomalyDetectionConfig) -> Self {
        Self {
            config,
            metric_history: HashMap::new(),
            last_detection_time: Utc::now(),
            previous_alerts: Vec::new(),
        }
    }

    /// Add a metric to the historical data used for detection
    pub fn add_metric(&mut self, metric: AnalyticsMetric) {
        if self.config.monitored_metrics.contains(&metric.metric_name) {
            // Check if the metric matches tag filters
            if let Some(tag_filters) = &self.config.tag_filters {
                let mut all_filters_match = true;
                for (key, value) in tag_filters {
                    if !metric.tags.get(key).map_or(false, |v| v == value) {
                        all_filters_match = false;
                        break;
                    }
                }
                if !all_filters_match {
                    return;
                }
            }

            // Add the metric to history
            self.metric_history
                .entry(metric.metric_name.clone())
                .or_insert_with(Vec::new)
                .push(metric);
        }
    }

    /// Check for anomalies in all monitored metrics
    pub fn detect_anomalies(&mut self) -> Vec<AnomalyAlert> {
        let now = Utc::now();
        let detection_interval = Duration::seconds(self.config.detection_interval_seconds as i64);
        
        // Only run detection if enough time has passed
        if now - self.last_detection_time < detection_interval {
            return Vec::new();
        }
        
        self.last_detection_time = now;
        let mut alerts = Vec::new();
        
        // Process each metric type
        for metric_name in &self.config.monitored_metrics {
            if let Some(metrics) = self.metric_history.get(metric_name) {
                // Skip if we don't have enough data points
                if metrics.len() < self.config.min_data_points {
                    continue;
                }
                
                // Detect anomalies based on the configured method
                let anomaly_alerts = match &self.config.detection_type {
                    AnomalyDetectionType::ZScore(threshold) => {
                        self.detect_zscore_anomalies(metrics, *threshold)
                    }
                    AnomalyDetectionType::MovingAverage { window_size, deviation_percent } => {
                        self.detect_moving_average_anomalies(metrics, *window_size, *deviation_percent)
                    }
                    AnomalyDetectionType::RateOfChange(threshold) => {
                        self.detect_rate_of_change_anomalies(metrics, *threshold)
                    }
                };
                
                // Filter out suppressed alerts
                let filtered_alerts = if let Some(suppression_window) = self.config.alert_suppression_window {
                    self.filter_suppressed_alerts(anomaly_alerts, suppression_window)
                } else {
                    anomaly_alerts
                };
                
                // Add unique alerts to the result
                for alert in filtered_alerts {
                    self.previous_alerts.push(alert.clone());
                    alerts.push(alert);
                }
            }
        }
        
        // Send alerts through configured channels
        self.send_alerts(&alerts);
        
        alerts
    }
    
    // Detect anomalies using Z-score (standard deviations from mean)
    fn detect_zscore_anomalies(&self, metrics: &[AnalyticsMetric], threshold: f64) -> Vec<AnomalyAlert> {
        let mut alerts = Vec::new();
        
        // Only process recent metrics (last 100 points)
        let recent_metrics = if metrics.len() > 100 {
            &metrics[metrics.len() - 100..]
        } else {
            metrics
        };
        
        // Calculate mean and standard deviation
        let values: Vec<f64> = recent_metrics.iter().map(|m| m.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        // Check most recent metrics for anomalies
        for metric in recent_metrics.iter().rev().take(5) {
            let z_score = (metric.value - mean).abs() / std_dev;
            if z_score > threshold && std_dev > 0.0 {
                let alert = AnomalyAlert {
                    timestamp: Utc::now(),
                    metric_name: metric.metric_name.clone(),
                    actual_value: metric.value,
                    expected_value: Some(mean),
                    deviation: z_score,
                    tags: metric.tags.clone(),
                    description: format!(
                        "Anomaly detected: {} is {:.2} standard deviations from mean ({:.2})",
                        metric.value, z_score, mean
                    ),
                    severity: if z_score > threshold * 2.0 {
                        AnomalySeverity::Critical
                    } else if z_score > threshold * 1.5 {
                        AnomalySeverity::Warning
                    } else {
                        AnomalySeverity::Info
                    },
                };
                alerts.push(alert);
            }
        }
        
        alerts
    }
    
    // Detect anomalies using moving average
    fn detect_moving_average_anomalies(
        &self, 
        metrics: &[AnalyticsMetric], 
        window_size: usize, 
        deviation_percent: f64
    ) -> Vec<AnomalyAlert> {
        let mut alerts = Vec::new();
        
        if metrics.len() < window_size + 1 {
            return alerts;
        }
        
        // Process each point with its moving average window
        for i in window_size..metrics.len() {
            let window = &metrics[i - window_size..i];
            let moving_avg = window.iter().map(|m| m.value).sum::<f64>() / window_size as f64;
            let current_value = metrics[i].value;
            
            // Calculate percentage deviation
            let percent_diff = ((current_value - moving_avg) / moving_avg).abs() * 100.0;
            
            if percent_diff > deviation_percent {
                let alert = AnomalyAlert {
                    timestamp: Utc::now(),
                    metric_name: metrics[i].metric_name.clone(),
                    actual_value: current_value,
                    expected_value: Some(moving_avg),
                    deviation: percent_diff,
                    tags: metrics[i].tags.clone(),
                    description: format!(
                        "Anomaly detected: {} is {:.2}% different from moving average of {:.2}",
                        current_value, percent_diff, moving_avg
                    ),
                    severity: if percent_diff > deviation_percent * 2.0 {
                        AnomalySeverity::Critical
                    } else if percent_diff > deviation_percent * 1.5 {
                        AnomalySeverity::Warning
                    } else {
                        AnomalySeverity::Info
                    },
                };
                alerts.push(alert);
            }
        }
        
        alerts
    }
    
    // Detect anomalies by looking at rate of change (first derivative)
    fn detect_rate_of_change_anomalies(&self, metrics: &[AnalyticsMetric], threshold: f64) -> Vec<AnomalyAlert> {
        let mut alerts = Vec::new();
        
        if metrics.len() < 3 {
            return alerts;
        }
        
        // Calculate rates of change
        let mut rates = Vec::with_capacity(metrics.len() - 1);
        for i in 1..metrics.len() {
            let time_diff = (metrics[i].timestamp - metrics[i-1].timestamp).num_seconds() as f64;
            if time_diff > 0.0 {
                let rate = (metrics[i].value - metrics[i-1].value) / time_diff;
                rates.push(rate);
            }
        }
        
        // Calculate average rate of change
        let avg_rate = rates.iter().sum::<f64>() / rates.len() as f64;
        let variance = rates.iter()
            .map(|&r| (r - avg_rate).powi(2))
            .sum::<f64>() / rates.len() as f64;
        let std_dev = variance.sqrt();
        
        // Check for anomalous rates of change
        for i in 1..rates.len() {
            let rate_diff = (rates[i] - avg_rate).abs();
            let z_score = if std_dev > 0.0 { rate_diff / std_dev } else { 0.0 };
            
            if z_score > threshold {
                let alert = AnomalyAlert {
                    timestamp: Utc::now(),
                    metric_name: metrics[i+1].metric_name.clone(),
                    actual_value: rates[i],
                    expected_value: Some(avg_rate),
                    deviation: z_score,
                    tags: metrics[i+1].tags.clone(),
                    description: format!(
                        "Rate of change anomaly: change rate of {:.2} is {:.2} standard deviations from average",
                        rates[i], z_score
                    ),
                    severity: if z_score > threshold * 2.0 {
                        AnomalySeverity::Critical
                    } else if z_score > threshold * 1.5 {
                        AnomalySeverity::Warning
                    } else {
                        AnomalySeverity::Info
                    },
                };
                alerts.push(alert);
            }
        }
        
        alerts
    }
    
    // Filter out repeat alerts within the suppression window
    fn filter_suppressed_alerts(&self, alerts: Vec<AnomalyAlert>, suppression_window: u64) -> Vec<AnomalyAlert> {
        let suppression_duration = Duration::seconds(suppression_window as i64);
        let now = Utc::now();
        
        alerts.into_iter().filter(|alert| {
            // Check if we've seen a similar alert recently
            !self.previous_alerts.iter().any(|prev| {
                prev.metric_name == alert.metric_name 
                && prev.severity == alert.severity
                && now - prev.timestamp < suppression_duration
            })
        }).collect()
    }
    
    // Send alerts through the configured channel
    fn send_alerts(&self, alerts: &[AnomalyAlert]) {
        for alert in alerts {
            match &self.config.alert_channel.channel_type {
                AlertChannelType::Log => {
                    match alert.severity {
                        AnomalySeverity::Info => info!("{}", alert.description),
                        AnomalySeverity::Warning => warn!("{}", alert.description),
                        AnomalySeverity::Critical => warn!("CRITICAL: {}", alert.description),
                    }
                },
                AlertChannelType::Webhook => {
                    // Logic to send to webhook would go here
                    if let Some(url) = self.config.alert_channel.config.get("url") {
                        info!("Would send alert to webhook {}: {}", url, alert.description);
                        // In a real implementation, this would make an HTTP request
                    }
                },
                AlertChannelType::Email => {
                    // Logic to send email would go here
                    if let Some(recipient) = self.config.alert_channel.config.get("recipient") {
                        info!("Would send alert to email {}: {}", recipient, alert.description);
                        // In a real implementation, this would send an email
                    }
                },
                AlertChannelType::Slack => {
                    // Logic to send to Slack would go here
                    if let Some(webhook) = self.config.alert_channel.config.get("webhook_url") {
                        info!("Would send alert to Slack webhook {}: {}", webhook, alert.description);
                        // In a real implementation, this would post to Slack
                    }
                },
            }
        }
    }
    
    /// Clear old metrics from history to prevent memory growth
    pub fn prune_history(&mut self, max_age: Duration) {
        let threshold = Utc::now() - max_age;
        
        for metrics in self.metric_history.values_mut() {
            metrics.retain(|m| m.timestamp >= threshold);
        }
        
        // Also prune old alerts
        self.previous_alerts.retain(|a| a.timestamp >= threshold);
    }
} 