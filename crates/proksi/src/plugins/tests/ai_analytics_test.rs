#[cfg(test)]
mod ai_analytics_tests {
    use crate::plugins::ai_analytics::{
        AiAnalytics, AiAnalyticsConfig, AnalyticsStorageType, AggregationType,
        AnomalyDetectionConfig, AnomalyDetectionType, AlertChannelConfig, AlertChannelType
    };
    use std::collections::HashMap;
    use pingora::proxy::Session;
    use crate::config::RoutePlugin;
    use crate::proxy_server::https_proxy::RouterContext;
    use chrono::{Utc, Duration};
    use serde_json::json;

    // Create a test analytics configuration
    fn create_test_config() -> AiAnalyticsConfig {
        let mut alert_thresholds = HashMap::new();
        alert_thresholds.insert("error_rate".to_string(), 0.05);
        alert_thresholds.insert("response_latency".to_string(), 1000.0);

        AiAnalyticsConfig {
            metrics_enabled: true,
            storage_type: AnalyticsStorageType::InMemory,
            retention_days: 30,
            sampling_rate: 1.0,  // Always sample in tests
            alert_thresholds,
            dashboard_enabled: true,
            dashboard_path: Some("/ai-analytics".to_string()),
            anomaly_detection: Some(create_test_anomaly_config()),
        }
    }
    
    // Create a test anomaly detection configuration
    fn create_test_anomaly_config() -> AnomalyDetectionConfig {
        let mut alert_config = HashMap::new();
        alert_config.insert("webhook_url".to_string(), "https://example.com/webhook".to_string());
        
        AnomalyDetectionConfig {
            detection_type: AnomalyDetectionType::ZScore(3.0),
            min_data_points: 10,
            monitored_metrics: vec![
                "response_latency".to_string(),
                "error_rate".to_string(),
                "total_tokens".to_string(),
            ],
            detection_interval_seconds: 60,
            tag_filters: None,
            alert_suppression_window: Some(300), // 5 minutes
            alert_channel: AlertChannelConfig {
                channel_type: AlertChannelType::Log,
                config: alert_config,
            },
        }
    }

    #[tokio::test]
    async fn test_analytics_config() {
        let config = create_test_config();
        
        assert!(config.metrics_enabled);
        assert!(matches!(config.storage_type, AnalyticsStorageType::InMemory));
        assert_eq!(config.retention_days, 30);
        assert_eq!(config.sampling_rate, 1.0);
        assert_eq!(config.alert_thresholds.len(), 2);
        assert_eq!(config.alert_thresholds.get("error_rate"), Some(&0.05));
        assert!(config.dashboard_enabled);
        assert_eq!(config.dashboard_path, Some("/ai-analytics".to_string()));
        assert!(config.anomaly_detection.is_some());
    }

    #[tokio::test]
    async fn test_anomaly_detection_config() {
        let anomaly_config = create_test_anomaly_config();
        
        // Test configuration values
        match anomaly_config.detection_type {
            AnomalyDetectionType::ZScore(threshold) => {
                assert_eq!(threshold, 3.0);
            },
            _ => panic!("Expected ZScore detection type"),
        }
        
        assert_eq!(anomaly_config.min_data_points, 10);
        assert_eq!(anomaly_config.monitored_metrics.len(), 3);
        assert!(anomaly_config.monitored_metrics.contains(&"response_latency".to_string()));
        assert_eq!(anomaly_config.detection_interval_seconds, 60);
        assert_eq!(anomaly_config.alert_suppression_window, Some(300));
        assert!(matches!(anomaly_config.alert_channel.channel_type, AlertChannelType::Log));
    }

    #[tokio::test]
    async fn test_analytics_storage_type_from_str() {
        assert!(matches!(
            AnalyticsStorageType::from_str("inmemory").unwrap(),
            AnalyticsStorageType::InMemory
        ));
        
        assert!(matches!(
            AnalyticsStorageType::from_str("redis").unwrap(),
            AnalyticsStorageType::Redis
        ));
        
        assert!(matches!(
            AnalyticsStorageType::from_str("postgres").unwrap(),
            AnalyticsStorageType::Postgres
        ));
        
        assert!(matches!(
            AnalyticsStorageType::from_str("custom").unwrap(),
            AnalyticsStorageType::Custom(s) if s == "custom"
        ));
    }

    #[tokio::test]
    async fn test_dashboard_rendering() {
        let analytics = AiAnalytics::new();
        let config = create_test_config();
        
        let dashboard_html = analytics.handle_dashboard_request(&config).unwrap();
        
        // Check if dashboard HTML contains expected elements
        assert!(dashboard_html.contains("Proksi AI Analytics Dashboard"));
        assert!(dashboard_html.contains("Request Count"));
        assert!(dashboard_html.contains("Response Time"));
        assert!(dashboard_html.contains("Error Rate"));
        assert!(dashboard_html.contains("Token Usage"));
        assert!(dashboard_html.contains("LLM Provider Breakdown"));
        assert!(dashboard_html.contains("Recent Requests"));
        
        // Check if anomaly detection elements are present
        assert!(dashboard_html.contains("Anomaly Detection"));
        assert!(dashboard_html.contains("Recent Anomalies"));
        assert!(dashboard_html.contains("anomaly-chart"));
    }

    #[tokio::test]
    async fn test_plugin_integration() {
        let analytics = AiAnalytics::new();
        let mut session = Session::new();
        let mut ctx = RouterContext {
            host: "test.example.com".to_string(),
            path: "/ai".to_string(),
            extensions: HashMap::new(),
            metadata: HashMap::new(),
            route_container: crate::proxy_server::https_proxy::RouteContainer {
                plugins: HashMap::new(),
            },
        };
        
        let plugin = RoutePlugin {
            name: "ai_analytics".to_string(),
            config: Some({
                let mut map = HashMap::new();
                map.insert("config_name".into(), serde_json::Value::String("test_config".to_string()));
                map
            }),
        };
        
        // Test request filter
        let result = analytics.request_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
        
        // Test response filter
        let result = analytics.response_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metric_collection() {
        let mut analytics = AiAnalytics::new();
        let config = create_test_config();
        let config_name = "test_config".to_string();
        
        // Add the config to the analytics instance
        let mut configs = HashMap::new();
        configs.insert(config_name.clone(), config.clone());
        analytics.config = std::sync::Arc::new(configs);
        
        // Initialize storage
        analytics.initialize_storage(&config_name, &config).await.unwrap();
        
        // Create a session with LLM-specific headers
        let mut session = Session::new();
        session.req_header_mut().append_header("x-llm-provider", "openai").unwrap();
        session.req_header_mut().append_header("x-llm-model", "gpt-4").unwrap();
        
        // Process request
        analytics.process_request(&mut session, &config).await.unwrap();
        
        // Set response headers for token metrics
        session.resp_header_mut().append_header("x-completion-tokens", "500").unwrap();
        session.resp_header_mut().append_header("x-prompt-tokens", "200").unwrap();
        session.resp_header_mut().append_header("x-total-tokens", "700").unwrap();
        
        // Process response
        analytics.process_response(&mut session, &config, &config_name).await.unwrap();
        
        // Check if anomaly detector exists
        assert!(analytics.anomaly_detectors.contains_key(&config_name));
    }

    #[tokio::test]
    async fn test_anomaly_detection_integration() {
        let mut analytics = AiAnalytics::new();
        let config = create_test_config();
        let config_name = "test_config".to_string();
        
        // Add the config to the analytics instance
        let mut configs = HashMap::new();
        configs.insert(config_name.clone(), config.clone());
        analytics.config = std::sync::Arc::new(configs);
        
        // Initialize storage and anomaly detector
        analytics.initialize_storage(&config_name, &config).await.unwrap();
        
        // Verify anomaly detector was created
        assert!(analytics.anomaly_detectors.contains_key(&config_name));
        
        // Create sessions with varying latencies to potentially trigger anomaly detection
        for i in 0..15 {
            let mut session = Session::new();
            
            // Normal latencies for first 10 requests
            let latency = if i < 10 {
                100.0 + (i as f64 * 10.0) // 100-190ms
            } else {
                // Anomalous latencies for the last 5 (to potentially trigger detection)
                500.0 + (i as f64 * 100.0) // 500-900ms
            };
            
            // Set latency on session
            session.response_time = std::time::Duration::from_millis(latency as u64);
            
            // Process response to add metrics to the anomaly detector
            analytics.process_response(&mut session, &config, &config_name).await.unwrap();
        }
        
        // Prune history (testing this functionality too)
        analytics.prune_metric_history(&config_name).await.unwrap();
    }

    #[tokio::test]
    async fn test_dashboard_request_handling() {
        let mut analytics = AiAnalytics::new();
        let config = create_test_config();
        let config_name = "test_config".to_string();
        
        // Add the config to the analytics instance
        let mut configs = HashMap::new();
        configs.insert(config_name.clone(), config.clone());
        analytics.config = std::sync::Arc::new(configs);
        
        // Create a session with a dashboard request path
        let mut session = Session::new();
        
        // Manually create a URI for the dashboard
        session.req_header_mut().uri = "/ai-analytics".parse().unwrap();
        
        let mut ctx = RouterContext {
            host: "test.example.com".to_string(),
            path: "/ai-analytics".to_string(),
            extensions: HashMap::new(),
            metadata: HashMap::new(),
            route_container: crate::proxy_server::https_proxy::RouteContainer {
                plugins: HashMap::new(),
            },
        };
        
        let plugin = RoutePlugin {
            name: "ai_analytics".to_string(),
            config: Some({
                let mut map = HashMap::new();
                map.insert("config_name".into(), serde_json::Value::String("test_config".to_string()));
                map
            }),
        };
        
        // Test the dashboard request handling
        let result = analytics.request_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
    }
} 