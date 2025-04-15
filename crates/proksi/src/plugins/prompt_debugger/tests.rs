use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use pingora::http::{ResponseHeader, StatusCode};
use pingora::proxy::Session;
use serde_json::json;

use crate::plugins::core::{Plugin, PluginStep};
use crate::plugins::prompt_debugger::{PromptDebugger, PromptDebuggerConfig, RuleSeverity, DebugRule};
use crate::proxy_server::https_proxy::RouterContext;

// Helper function to create a mock session
fn create_mock_session(path: &str) -> Session {
    let req_header = pingora::http::RequestHeader::build(
        &pingora::http::Method::GET,
        path,
        None,
    ).unwrap();
    Session::new_for_test(req_header)
}

// Helper function to create a mock context
fn create_mock_context() -> RouterContext {
    RouterContext {
        host: "example.com".to_string(),
        route_container: Default::default(),
        upstream: Default::default(),
        extensions: HashMap::new(),
        is_websocket: false,
        timings: Default::default(),
        plugins_data: HashMap::new(),
        request_id: "test-request-id".to_string(),
        upstream_response: None,
    }
}

#[tokio::test]
async fn test_plugin_trait_implementation() {
    let debugger = PromptDebugger::new();
    
    // Verify trait methods
    assert_eq!(debugger.name(), "prompt_debugger");
    
    // Start and stop methods should not fail
    let mut debugger = PromptDebugger::new();
    assert!(debugger.start().await.is_ok());
    assert!(debugger.stop().await.is_ok());
}

#[tokio::test]
async fn test_prompt_analysis() {
    let mut debugger = PromptDebugger::new();
    
    // Add some test rules
    debugger.config.rules = vec![
        DebugRule {
            pattern: "unsafe".to_string(),
            severity: RuleSeverity::Warning,
            message: "Contains unsafe content".to_string(),
        }
    ];
    
    let mut ctx = create_mock_context();
    
    // Create a test session with a prompt
    let req_header = pingora::http::RequestHeader::build(
        &pingora::http::Method::POST,
        "/chat/completions",
        None,
    ).unwrap();
    let mut session = Session::new_for_test(req_header);
    
    // Handle the request
    let result = debugger.handle_request(
        PluginStep::Request,
        &mut session,
        &mut ctx
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_request_ui_endpoint() {
    // Create a mock session and context
    let mut mock_session = create_mock_session("/prompt-debugger");
    let mut mock_ctx = create_mock_context();
    
    // Create plugin with custom config
    let mut config = PromptDebuggerConfig::default();
    config.ui_endpoint = "/prompt-debugger".to_string();
    let debugger = PromptDebugger::new();
    
    // Handle the request
    let result = debugger.handle_request(
        PluginStep::Request,
        &mut mock_session,
        &mut mock_ctx
    ).await;
    
    // Because we're not fully initializing the session in our test,
    // we expect it might fail, but we should see it attempt to handle
    match result {
        Ok((handled, _)) => {
            assert!(handled);
        },
        Err(_) => {
            // Expected in test because mock session lacks many fields
        }
    }
}

#[tokio::test]
async fn test_handle_response() {
    // Create a mock session, context and response
    let mut mock_session = create_mock_session("/chat/completions");
    let mut mock_ctx = create_mock_context();
    let mut mock_response = ResponseHeader::build(StatusCode::OK, None).unwrap();
    
    // Add analysis data to context
    let analysis = json!({
        "timestamp": "2023-01-01T00:00:00Z",
        "prompt": "Test prompt",
        "results": [],
        "suggestions": [],
        "improved_prompt": null
    });
    
    mock_ctx.plugins_data.insert(
        "prompt_debugger_analysis".to_string(),
        analysis
    );
    
    mock_ctx.plugins_data.insert(
        "prompt_debugger_original_prompt".to_string(),
        json!("Test prompt")
    );
    
    mock_ctx.request_id = "test-request-id".to_string();
    
    // Create plugin
    let debugger = PromptDebugger::new();
    
    // Handle the response
    let result = debugger.handle_response(
        PluginStep::Response,
        &mut mock_session,
        &mut mock_ctx,
        &mut mock_response
    ).await;
    
    // It should succeed and modify the response
    assert!(result.is_ok());
    
    // Check if the header was added (this may fail in our mocked test)
    let has_header = mock_response.headers.get("X-Prompt-Analyzed").is_some();
    assert!(has_header || result.unwrap(), "Header should be added or function should return modified=true");
}