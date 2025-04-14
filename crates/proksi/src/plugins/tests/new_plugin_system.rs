use std::sync::Arc;
use anyhow::Result;
use serde_json::json;
use async_trait::async_trait;

use crate::plugins::core::{
    Plugin, PluginType, PluginStep, PluginError, 
    PluginFactory, PluginMetadata, PluginRegistry
};
use crate::plugins::executor::{execute_plugins, initialize_plugins, shutdown_plugins};
use crate::proxy_server::https_proxy::{RouterContext, HttpResponse};

#[derive(Debug)]
struct TestPlugin {
    name: &'static str,
    metadata: PluginMetadata,
    invocations: std::sync::atomic::AtomicUsize,
}

impl TestPlugin {
    fn new(name: &'static str, priority: i32) -> Self {
        Self {
            name,
            metadata: PluginMetadata {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                priority,
                plugin_type: PluginType::Native,
                description: format!("Test plugin {} for integration tests", name),
                author: "Test Author".to_string(),
                homepage: Some("https://example.com".to_string()),
            },
            invocations: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    
    fn invocation_count(&self) -> usize {
        self.invocations.load(std::sync::atomic::Ordering::SeqCst)
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
        _session: &mut pingora::proxy::Session,
        _ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        self.invocations.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        // Plugin with name "block" will block the request
        if self.name == "block" && step == PluginStep::Request {
            return Ok((true, None));
        }
        
        Ok((false, None))
    }
    
    async fn start(&mut self) -> Result<(), PluginError> {
        println!("Starting plugin {}", self.name);
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), PluginError> {
        println!("Stopping plugin {}", self.name);
        Ok(())
    }
}

struct TestPluginFactory {
    name: &'static str,
    priority: i32,
}

impl TestPluginFactory {
    fn new(name: &'static str, priority: i32) -> Self {
        Self { name, priority }
    }
}

impl PluginFactory for TestPluginFactory {
    fn create(&self, _config: &serde_json::Value) -> Result<Box<dyn Plugin>, PluginError> {
        Ok(Box::new(TestPlugin::new(self.name, self.priority)))
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name.to_string(),
            version: "1.0.0".to_string(),
            priority: self.priority,
            plugin_type: PluginType::Native,
            description: format!("Test plugin {} for integration tests", self.name),
            author: "Test Author".to_string(),
            homepage: Some("https://example.com".to_string()),
        }
    }
}

#[tokio::test]
async fn test_plugin_registry_and_execution() -> Result<()> {
    // Create plugin registry
    let mut registry = PluginRegistry::new();
    
    // Register test plugins with different priorities
    registry.register_factory(Box::new(TestPluginFactory::new("first", 10)))?;
    registry.register_factory(Box::new(TestPluginFactory::new("second", 20)))?;
    registry.register_factory(Box::new(TestPluginFactory::new("third", 30)))?;
    registry.register_factory(Box::new(TestPluginFactory::new("block", 15)))?;
    
    // Verify plugins are registered
    let plugins = registry.list_plugins();
    assert_eq!(plugins.len(), 4);
    
    // Create plugin instances
    let config = json!({});
    let first_plugin = registry.get_or_create_plugin("first", &config).await?;
    let second_plugin = registry.get_or_create_plugin("second", &config).await?;
    let third_plugin = registry.get_or_create_plugin("third", &config).await?;
    let block_plugin = registry.get_or_create_plugin("block", &config).await?;
    
    // Verify instances are created
    assert_eq!(first_plugin.name(), "first");
    assert_eq!(second_plugin.name(), "second");
    assert_eq!(third_plugin.name(), "third");
    assert_eq!(block_plugin.name(), "block");
    
    // Test execute_plugins with all plugins
    let mut all_plugins = vec![
        first_plugin.clone(),
        second_plugin.clone(), 
        third_plugin.clone(),
        block_plugin.clone(),
    ];
    
    // Initialize plugins
    initialize_plugins(&mut all_plugins).await?;
    
    // Create context and session for testing
    let mut ctx = RouterContext::default();
    let mut session = pingora::proxy::Session::new_dummy();
    
    // Execute plugins at EarlyRequest step (all should run)
    let (handled, _) = execute_plugins(
        PluginStep::EarlyRequest,
        &mut session,
        &mut ctx,
        &all_plugins,
    ).await?;
    
    assert!(!handled);
    
    // All plugins should have been invoked once
    assert_eq!(
        as_test_plugin(&first_plugin).invocation_count(), 
        1
    );
    assert_eq!(
        as_test_plugin(&second_plugin).invocation_count(), 
        1
    );
    assert_eq!(
        as_test_plugin(&third_plugin).invocation_count(), 
        1
    );
    assert_eq!(
        as_test_plugin(&block_plugin).invocation_count(), 
        1
    );
    
    // Execute plugins at Request step - block plugin should halt execution
    let (handled, _) = execute_plugins(
        PluginStep::Request,
        &mut session,
        &mut ctx,
        &all_plugins,
    ).await?;
    
    assert!(handled);
    
    // first plugin should have been invoked since its priority is 10 (before block at 15)
    assert_eq!(
        as_test_plugin(&first_plugin).invocation_count(), 
        2
    );
    
    // block plugin should have been invoked
    assert_eq!(
        as_test_plugin(&block_plugin).invocation_count(), 
        2
    );
    
    // second and third plugins should not have been invoked since block plugin returned handled=true
    assert_eq!(
        as_test_plugin(&second_plugin).invocation_count(), 
        1
    );
    assert_eq!(
        as_test_plugin(&third_plugin).invocation_count(), 
        1
    );
    
    // Shutdown plugins
    shutdown_plugins(&mut all_plugins).await?;
    
    Ok(())
}

// Helper function to downcast Arc<dyn Plugin> to &TestPlugin
fn as_test_plugin(plugin: &Arc<dyn Plugin>) -> &TestPlugin {
    unsafe {
        // This is safe because we know the concrete type is TestPlugin
        let ptr = Arc::as_ptr(plugin) as *const TestPlugin;
        &*ptr
    }
}

#[tokio::test]
async fn test_plugin_step_serialization() {
    use strum::ParseError;
    use std::str::FromStr;
    
    // Test conversions between string and PluginStep
    assert_eq!(PluginStep::from_str("early_request").unwrap(), PluginStep::EarlyRequest);
    assert_eq!(PluginStep::from_str("request").unwrap(), PluginStep::Request);
    assert_eq!(PluginStep::from_str("proxy_upstream").unwrap(), PluginStep::ProxyUpstream);
    assert_eq!(PluginStep::from_str("response").unwrap(), PluginStep::Response);
    assert_eq!(PluginStep::from_str("response_body").unwrap(), PluginStep::ResponseBody);
    assert_eq!(PluginStep::from_str("log").unwrap(), PluginStep::Log);
    
    // Test invalid string
    assert!(matches!(PluginStep::from_str("invalid"), Err(ParseError::VariantNotFound)));
    
    // Test to_string
    assert_eq!(PluginStep::EarlyRequest.to_string(), "early_request");
    assert_eq!(PluginStep::Request.to_string(), "request");
    assert_eq!(PluginStep::ProxyUpstream.to_string(), "proxy_upstream");
    assert_eq!(PluginStep::Response.to_string(), "response");
    assert_eq!(PluginStep::ResponseBody.to_string(), "response_body");
    assert_eq!(PluginStep::Log.to_string(), "log");
} 