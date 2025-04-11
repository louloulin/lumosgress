#[cfg(test)]
mod ai_request_builder_tests {
    use std::collections::HashMap;
    use serde_json::json;

    use crate::{
        config::RoutePlugin,
        plugins::ai_request_builder::{AiRequestBuilder, AiRequestBuilderConfig},
        proxy_server::https_proxy::{RouteContainer, RouterContext},
    };
    use std::borrow::Cow;
    use pingora::proxy::Session;

    #[tokio::test]
    async fn test_default_config() {
        let config = AiRequestBuilderConfig::default();
        
        // Verify the default configuration
        assert_eq!(config.templates.len(), 2);
        assert_eq!(config.ui_endpoint, "/ai-request-builder");
        assert!(config.enable_api);
        assert!(config.save_history);
        assert_eq!(config.max_history_entries, Some(100));
        
        // Check template details
        assert_eq!(config.templates[0].name, "OpenAI Chat");
        assert_eq!(config.templates[1].name, "Anthropic Claude");
    }

    #[tokio::test]
    async fn test_parse_config() {
        let builder = AiRequestBuilder::new();
        
        // Create a test plugin config
        let mut config_map = HashMap::new();
        config_map.insert(
            Cow::from("templates"),
            json!([
                {
                    "name": "Test Template",
                    "endpoint": "api.test.com/v1/chat",
                    "method": "POST",
                    "headers": {
                        "Content-Type": "application/json",
                        "Authorization": "Bearer test_key"
                    },
                    "body_template": "{}",
                    "description": "Test template"
                }
            ]),
        );
        
        let plugin = RoutePlugin {
            name: Cow::from("ai_request_builder"),
            config: Some(config_map),
        };
        
        // Parse the config
        let parsed_config = builder.parse_config(&plugin).await.unwrap();
        
        // Verify parsed config
        assert_eq!(parsed_config.templates.len(), 1);
        assert_eq!(parsed_config.templates[0].name, "Test Template");
        assert_eq!(parsed_config.templates[0].endpoint, "api.test.com/v1/chat");
        assert_eq!(parsed_config.templates[0].method, "POST");
        
        // Verify headers
        assert_eq!(parsed_config.templates[0].headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(parsed_config.templates[0].headers.get("Authorization").unwrap(), "Bearer test_key");
    }

    #[tokio::test]
    async fn test_plugin_integration() {
        let builder = AiRequestBuilder::new();
        let mut session = Session::new();
        let mut ctx = RouterContext {
            host: "test.example.com".to_string(),
            path: "/ai-request-builder".to_string(),
            extensions: HashMap::new(),
            metadata: HashMap::new(),
            route_container: RouteContainer {
                plugins: HashMap::new(),
            },
        };
        
        let plugin = RoutePlugin {
            name: Cow::from("ai_request_builder"),
            config: None,
        };
        
        // Test request filter for UI endpoint
        let result = builder.request_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
        
        // The plugin should handle the request since it matches the UI endpoint
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_non_ui_request() {
        let builder = AiRequestBuilder::new();
        let mut session = Session::new();
        let mut ctx = RouterContext {
            host: "test.example.com".to_string(),
            path: "/some-other-path".to_string(),
            extensions: HashMap::new(),
            metadata: HashMap::new(),
            route_container: RouteContainer {
                plugins: HashMap::new(),
            },
        };
        
        let plugin = RoutePlugin {
            name: Cow::from("ai_request_builder"),
            config: None,
        };
        
        // Test request filter for non-UI endpoint
        let result = builder.request_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
        
        // The plugin should not handle the request since it doesn't match the UI endpoint
        assert!(!result.unwrap());
    }
} 