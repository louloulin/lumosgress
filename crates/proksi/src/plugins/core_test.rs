#[cfg(test)]
mod tests {

    use anyhow::Result;
    use serde_json::{json, Value};
    use async_trait::async_trait;
    use pingora::proxy::Session;
    use crate::plugins::core::{Plugin, PluginType, PluginStep, PluginError, PluginMetadata, PluginRegistry, PluginFactory};
    use crate::proxy_server::HttpResponse;
    use crate::proxy_server::https_proxy::RouterContext;

    // Simple test plugin implementation
    #[derive(Debug)]
    struct TestPlugin {
        name: &'static str,
        metadata: PluginMetadata,
        handled_requests: std::sync::atomic::AtomicUsize,
    }

    impl TestPlugin {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                metadata: PluginMetadata {
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    priority: 10,
                    plugin_type: PluginType::Native,
                    description: "Test plugin for unit tests".to_string(),
                    author: "Test Author".to_string(),
                    homepage: Some("https://example.com".to_string()),
                },
                handled_requests: std::sync::atomic::AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            self.name
        }

        fn metadata(&self) -> PluginMetadata {
            self.metadata.clone()
        }

        async fn handle_request(
            &self,
            step: PluginStep,
            _session: &mut Session,
            _ctx: &mut RouterContext,
        ) -> Result<(bool, Option<HttpResponse>)> {
            self.handled_requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            match step {
                PluginStep::Request => Ok((true, None)),
                _ => Ok((false, None)),
            }
        }
    }

    // Test factory for TestPlugin
    struct TestPluginFactory;

    impl PluginFactory for TestPluginFactory {
        fn create(&self, _config: &Value) -> Result<Box<dyn Plugin>, PluginError> {
            Ok(Box::new(TestPlugin::new("test_plugin")))
        }

        fn name(&self) -> &'static str {
            "test_plugin"
        }

        fn metadata(&self) -> PluginMetadata {
            PluginMetadata {
                name: "test_plugin".to_string(),
                version: "1.0.0".to_string(),
                priority: 10,
                plugin_type: PluginType::Native,
                description: "Test plugin for unit tests".to_string(),
                author: "Test Author".to_string(),
                homepage: Some("https://example.com".to_string()),
            }
        }
    }

    #[tokio::test]
    async fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();
        
        // Register a test plugin factory
        registry.register_factory(Box::new(TestPluginFactory)).unwrap();
        
        // List all plugins
        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "test_plugin");
        
        // Create a plugin instance
        let config = json!({
            "test_key": "test_value"
        });
        
        let plugin = registry.get_or_create_plugin("test_plugin", &config).await.unwrap();
        assert_eq!(plugin.name(), "test_plugin");
        
        // Try to get the same plugin (should use cached instance)
        let same_plugin = registry.get_or_create_plugin("test_plugin", &config).await.unwrap();
        assert_eq!(same_plugin.name(), "test_plugin");
        
        // Unload the plugin
        registry.unload_plugin("test_plugin").await.unwrap();
        
        // Try to unload a non-existent plugin
        let err = registry.unload_plugin("non_existent").await.unwrap_err();
        assert!(matches!(err, PluginError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_plugin_step_default() {
        assert_eq!(PluginStep::default(), PluginStep::Request);
    }

    #[tokio::test]
    async fn test_plugin_type_default() {
        assert_eq!(PluginType::default(), PluginType::Native);
    }

    #[tokio::test]
    async fn test_plugin_metadata() {
        let plugin = TestPlugin::new("test_plugin");
        let metadata = plugin.metadata();
        
        assert_eq!(metadata.name, "test_plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.priority, 10);
        assert_eq!(metadata.plugin_type, PluginType::Native);
        assert_eq!(metadata.description, "Test plugin for unit tests");
        assert_eq!(metadata.author, "Test Author");
        assert_eq!(metadata.homepage, Some("https://example.com".to_string()));
    }
} 