#[cfg(test)]
mod prompt_debugger_tests {
    use std::collections::HashMap;

    use crate::{
        config::RoutePlugin,
        plugins::prompt_debugger::{PromptDebugger, PromptDebuggerConfig, RuleSeverity},
        proxy_server::https_proxy::{RouteContainer, RouterContext},
    };
    use std::borrow::Cow;
    use pingora::proxy::Session;

    #[tokio::test]
    async fn test_default_config() {
        let config = PromptDebuggerConfig::default();
        
        // Verify the default configuration
        assert_eq!(config.ui_endpoint, "/prompt-debugger");
        assert!(config.enable_api);
        assert_eq!(config.max_history_entries, Some(100));
        assert!(!config.intercept_requests);
        
        // Check rules
        assert!(config.rules.len() > 0);
        assert!(config.rules.iter().any(|r| r.name == "System Role First"));
        assert!(config.rules.iter().any(|r| r.name == "Vague Language"));
    }

    #[tokio::test]
    async fn test_analyze_prompt() {
        let debugger = PromptDebugger::new();
        let config = PromptDebuggerConfig::default();
        
        // Test with a prompt that should match the vague language rule
        let prompt = "Tell me about some stuff and things.";
        let analysis = debugger.analyze_prompt(prompt, &config);
        
        // There should be a match for "vague language" rule
        let vague_rule_match = analysis.results.iter().find(|r| r.rule_name.contains("Vague") && r.matches);
        assert!(vague_rule_match.is_some());
        assert_eq!(vague_rule_match.unwrap().severity, RuleSeverity::Warning);
        
        // There should be suggestions
        assert!(!analysis.suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_extract_prompt_from_request() {
        let debugger = PromptDebugger::new();
        
        // Test OpenAI format
        let openai_json = r#"{
            "model": "gpt-4",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant"},
                {"role": "user", "content": "Tell me about AI"}
            ]
        }"#;
        
        let extracted = debugger.extract_prompt_from_request(openai_json);
        assert_eq!(extracted, Some("Tell me about AI".to_string()));
        
        // Test Google/Gemini format
        let google_json = r#"{
            "contents": [
                {
                    "parts": [
                        {"text": "Write a poem about nature"}
                    ]
                }
            ]
        }"#;
        
        let extracted = debugger.extract_prompt_from_request(google_json);
        assert_eq!(extracted, Some("Write a poem about nature".to_string()));
    }

    #[tokio::test]
    async fn test_plugin_ui_endpoint() {
        let debugger = PromptDebugger::new();
        let mut session = Session::new();
        let mut ctx = RouterContext {
            host: "test.example.com".to_string(),
            path: "/prompt-debugger".to_string(),
            extensions: HashMap::new(),
            metadata: HashMap::new(),
            route_container: RouteContainer {
                plugins: HashMap::new(),
            },
        };
        
        let plugin = RoutePlugin {
            name: Cow::from("prompt_debugger"),
            config: None,
        };
        
        // Test request filter for UI endpoint
        let result = debugger.request_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
        
        // The plugin should handle the request since it matches the UI endpoint
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_custom_rules() {
        use serde_json::json;
        
        let debugger = PromptDebugger::new();
        
        // Create plugin config with custom rules
        let mut config_map = HashMap::new();
        config_map.insert(
            Cow::from("rules"),
            json!([
                {
                    "name": "Custom Rule",
                    "description": "Test custom rule",
                    "pattern": "test pattern",
                    "suggestion": "This is a test suggestion",
                    "severity": "error"
                }
            ]),
        );
        
        let plugin = RoutePlugin {
            name: Cow::from("prompt_debugger"),
            config: Some(config_map),
        };
        
        // Parse config
        let config = debugger.parse_config(&plugin).await.unwrap();
        
        // Check custom rule
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].name, "Custom Rule");
        assert_eq!(config.rules[0].pattern, "test pattern");
        assert_eq!(config.rules[0].severity, RuleSeverity::Error);
        
        // Test analysis with the custom rule
        let prompt = "This contains test pattern here";
        let analysis = debugger.analyze_prompt(prompt, &config);
        
        assert!(analysis.results[0].matches);
        assert_eq!(analysis.results[0].severity, RuleSeverity::Error);
    }
} 