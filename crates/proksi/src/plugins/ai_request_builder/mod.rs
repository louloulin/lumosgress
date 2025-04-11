use std::{borrow::Cow, collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pingora::{http::{RequestHeader, ResponseHeader}, proxy::Session};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use bytes;
use http::StatusCode;

use crate::{config::RoutePlugin, proxy_server::https_proxy::RouterContext};
use super::MiddlewarePlugin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderTemplate {
    pub name: String,
    pub endpoint: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body_template: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequestBuilderConfig {
    pub templates: Vec<ProviderTemplate>,
    pub ui_endpoint: String,
    pub enable_api: bool,
    pub save_history: bool,
    pub max_history_entries: Option<usize>,
}

impl Default for AiRequestBuilderConfig {
    fn default() -> Self {
        Self {
            templates: vec![
                ProviderTemplate {
                    name: "OpenAI Chat".to_string(),
                    endpoint: "api.openai.com/v1/chat/completions".to_string(),
                    method: "POST".to_string(),
                    headers: {
                        let mut headers = HashMap::new();
                        headers.insert("Content-Type".to_string(), "application/json".to_string());
                        headers.insert("Authorization".to_string(), "Bearer ${OPENAI_API_KEY}".to_string());
                        headers
                    },
                    body_template: r#"{
  "model": "gpt-3.5-turbo",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "Hello, how are you?"
    }
  ],
  "temperature": 0.7
}"#.to_string(),
                    description: "OpenAI ChatGPT API for chat completions".to_string(),
                },
                ProviderTemplate {
                    name: "Anthropic Claude".to_string(),
                    endpoint: "api.anthropic.com/v1/messages".to_string(),
                    method: "POST".to_string(),
                    headers: {
                        let mut headers = HashMap::new();
                        headers.insert("Content-Type".to_string(), "application/json".to_string());
                        headers.insert("x-api-key".to_string(), "${ANTHROPIC_API_KEY}".to_string());
                        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
                        headers
                    },
                    body_template: r#"{
  "model": "claude-3-opus-20240229",
  "messages": [
    {
      "role": "user",
      "content": "Hello, Claude! How are you today?"
    }
  ],
  "max_tokens": 1000
}"#.to_string(),
                    description: "Anthropic Claude API for chat completions".to_string(),
                },
            ],
            ui_endpoint: "/ai-request-builder".to_string(),
            enable_api: true,
            save_history: true,
            max_history_entries: Some(100),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub provider: String,
    pub request: String,
    pub response: Option<String>,
    pub duration_ms: u64,
    pub status_code: u16,
}

pub struct AiRequestBuilder {
    config: Arc<Mutex<HashMap<String, AiRequestBuilderConfig>>>,
    request_history: Arc<Mutex<Vec<RequestHistoryEntry>>>,
}

impl AiRequestBuilder {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(HashMap::new())),
            request_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Parse a configuration from a plugin configuration
    async fn parse_config(&self, plugin: &RoutePlugin) -> Result<AiRequestBuilderConfig> {
        if let Some(config) = &plugin.config {
            if let Some(config_name) = config.get("config_name") {
                if let Some(config_name) = config_name.as_str() {
                    let configs = self.config.lock().await;
                    if let Some(config) = configs.get(config_name) {
                        return Ok(config.clone());
                    }
                }
            }

            // If custom templates are provided in the config, use them
            if let Some(templates) = config.get("templates") {
                if let Some(templates) = templates.as_array() {
                    let mut provider_templates = Vec::new();
                    for template in templates {
                        if let Some(template_obj) = template.as_object() {
                            if let (Some(name), Some(endpoint), Some(method), Some(body)) = (
                                template_obj.get("name").and_then(|v| v.as_str()),
                                template_obj.get("endpoint").and_then(|v| v.as_str()),
                                template_obj.get("method").and_then(|v| v.as_str()),
                                template_obj.get("body_template").and_then(|v| v.as_str()),
                            ) {
                                let mut headers = HashMap::new();
                                if let Some(header_obj) = template_obj.get("headers").and_then(|v| v.as_object()) {
                                    for (key, value) in header_obj {
                                        if let Some(value_str) = value.as_str() {
                                            headers.insert(key.clone(), value_str.to_string());
                                        }
                                    }
                                }

                                let description = template_obj
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                provider_templates.push(ProviderTemplate {
                                    name: name.to_string(),
                                    endpoint: endpoint.to_string(),
                                    method: method.to_string(),
                                    headers,
                                    body_template: body.to_string(),
                                    description,
                                });
                            }
                        }
                    }

                    let ui_endpoint = config
                        .get("ui_endpoint")
                        .and_then(|v| v.as_str())
                        .unwrap_or("/ai-request-builder")
                        .to_string();

                    let enable_api = config
                        .get("enable_api")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    let save_history = config
                        .get("save_history")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    let max_history_entries = config
                        .get("max_history_entries")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as usize);

                    return Ok(AiRequestBuilderConfig {
                        templates: provider_templates,
                        ui_endpoint,
                        enable_api,
                        save_history,
                        max_history_entries,
                    });
                }
            }
        }

        // Return default configuration if no specific config provided
        Ok(AiRequestBuilderConfig::default())
    }

    // Add an entry to the request history
    async fn add_history_entry(&self, entry: RequestHistoryEntry, config: &AiRequestBuilderConfig) {
        if !config.save_history {
            return;
        }

        let mut history = self.request_history.lock().await;
        history.push(entry);

        // Prune history if it exceeds max entries
        if let Some(max) = config.max_history_entries {
            if history.len() > max {
                history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                history.truncate(max);
            }
        }
    }

    // Serve the UI for the request builder
    async fn serve_ui(&self, session: &mut Session, config: &AiRequestBuilderConfig) -> Result<bool> {
        let html = self.generate_ui_html(config).await;
        let content_length = html.len();

        // Set response headers
        let mut response = ResponseHeader::build(StatusCode::OK, None)?;
        response.append_header("Content-Type", "text/html; charset=utf-8")?;
        response.append_header("Content-Length", content_length.to_string())?;

        session.write_response_header(Box::new(response), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(html)), true).await?;

        Ok(true)
    }

    // Generate the HTML for the request builder UI
    async fn generate_ui_html(&self, config: &AiRequestBuilderConfig) -> String {
        let mut template_options = String::new();
        for template in &config.templates {
            template_options.push_str(&format!(
                r#"<option value="{}">{}</option>"#,
                template.name, template.name
            ));
        }

        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AI Request Builder</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }
        h1, h2, h3 {
            color: #2c3e50;
        }
        .container {
            display: flex;
            gap: 20px;
        }
        .left-panel {
            flex: 1;
        }
        .right-panel {
            flex: 1;
        }
        select, textarea, input, button {
            width: 100%;
            padding: 8px;
            margin-bottom: 10px;
            border: 1px solid #ddd;
            border-radius: 4px;
        }
        textarea {
            min-height: 300px;
            font-family: monospace;
        }
        button {
            background-color: #4CAF50;
            color: white;
            border: none;
            cursor: pointer;
            padding: 10px;
        }
        button:hover {
            background-color: #45a049;
        }
        .response {
            margin-top: 20px;
            border: 1px solid #ddd;
            padding: 10px;
            border-radius: 4px;
            background-color: #f9f9f9;
            min-height: 300px;
            white-space: pre-wrap;
            font-family: monospace;
            overflow: auto;
        }
        .header-row {
            display: flex;
            margin-bottom: 5px;
        }
        .header-key {
            flex: 1;
            margin-right: 5px;
        }
        .header-value {
            flex: 2;
        }
        .remove-header {
            background-color: #f44336;
            color: white;
            border: none;
            padding: 5px 10px;
            margin-left: 5px;
            cursor: pointer;
            border-radius: 4px;
        }
        .add-header {
            background-color: #2196F3;
            width: auto;
        }
        #headers-container {
            margin-bottom: 10px;
        }
        .info {
            margin-top: 10px;
            color: #666;
        }
        .tabs {
            display: flex;
            margin-bottom: 20px;
            border-bottom: 1px solid #ddd;
        }
        .tab {
            padding: 10px 20px;
            cursor: pointer;
            background-color: #f1f1f1;
            border: 1px solid #ddd;
            border-bottom: none;
            margin-right: 5px;
            border-radius: 4px 4px 0 0;
        }
        .tab.active {
            background-color: white;
            border-bottom: 1px solid white;
            margin-bottom: -1px;
        }
        .tab-content {
            display: none;
        }
        .tab-content.active {
            display: block;
        }
        .history-item {
            padding: 10px;
            border: 1px solid #ddd;
            margin-bottom: 10px;
            border-radius: 4px;
            cursor: pointer;
        }
        .history-item:hover {
            background-color: #f1f1f1;
        }
        .history-time {
            font-size: 0.8em;
            color: #666;
        }
        .history-provider {
            font-weight: bold;
        }
    </style>
</head>
<body>
    <h1>AI Request Builder</h1>
    
    <div class="tabs">
        <div class="tab active" data-tab="request-builder">Request Builder</div>
        <div class="tab" data-tab="history">Request History</div>
    </div>
    
    <div id="request-builder" class="tab-content active">
        <div class="container">
            <div class="left-panel">
                <h2>Request</h2>
                
                <label for="template">Provider Template:</label>
                <select id="template">
                    TEMPLATE_OPTIONS_PLACEHOLDER
                </select>
                
                <label for="endpoint">Endpoint:</label>
                <input type="text" id="endpoint" placeholder="https://api.example.com/v1/completions">
                
                <label for="method">Method:</label>
                <select id="method">
                    <option value="GET">GET</option>
                    <option value="POST" selected>POST</option>
                    <option value="PUT">PUT</option>
                    <option value="DELETE">DELETE</option>
                </select>
                
                <h3>Headers</h3>
                <div id="headers-container"></div>
                <button class="add-header" id="add-header">Add Header</button>
                
                <label for="request-body">Request Body:</label>
                <textarea id="request-body" placeholder="{}"></textarea>
                
                <button id="send-request">Send Request</button>
            </div>
            
            <div class="right-panel">
                <h2>Response</h2>
                <div id="response-info" class="info"></div>
                <div id="response" class="response"></div>
            </div>
        </div>
    </div>
    
    <div id="history" class="tab-content">
        <h2>Request History</h2>
        <div id="history-container">
            <p>No request history available.</p>
        </div>
    </div>

    <script>
        // Initialize UI
        document.addEventListener('DOMContentLoaded', () => {
            // Set up tabs
            const tabs = document.querySelectorAll('.tab');
            tabs.forEach(tab => {
                tab.addEventListener('click', () => {
                    tabs.forEach(t => t.classList.remove('active'));
                    tab.classList.add('active');
                    
                    const tabContents = document.querySelectorAll('.tab-content');
                    tabContents.forEach(content => content.classList.remove('active'));
                    
                    const targetTab = tab.getAttribute('data-tab');
                    document.getElementById(targetTab).classList.add('active');
                    
                    if (targetTab === 'history') {
                        loadHistory();
                    }
                });
            });
            
            // Set up template selection
            const templateSelect = document.getElementById('template');
            templateSelect.addEventListener('change', () => {
                const selectedTemplate = templateSelect.value;
                // TODO: Load the selected template data
            });
            
            // Set up header management
            const addHeaderButton = document.getElementById('add-header');
            addHeaderButton.addEventListener('click', addHeaderRow);
            
            // Add initial header row
            addHeaderRow();
            
            // Set up send request button
            const sendRequestButton = document.getElementById('send-request');
            sendRequestButton.addEventListener('click', sendRequest);
        });
        
        function addHeaderRow() {
            const headersContainer = document.getElementById('headers-container');
            const headerRow = document.createElement('div');
            headerRow.className = 'header-row';
            
            const keyInput = document.createElement('input');
            keyInput.className = 'header-key';
            keyInput.placeholder = 'Header Name';
            
            const valueInput = document.createElement('input');
            valueInput.className = 'header-value';
            valueInput.placeholder = 'Header Value';
            
            const removeButton = document.createElement('button');
            removeButton.className = 'remove-header';
            removeButton.textContent = 'X';
            removeButton.addEventListener('click', () => {
                headersContainer.removeChild(headerRow);
            });
            
            headerRow.appendChild(keyInput);
            headerRow.appendChild(valueInput);
            headerRow.appendChild(removeButton);
            
            headersContainer.appendChild(headerRow);
        }
        
        async function sendRequest() {
            const endpoint = document.getElementById('endpoint').value.trim();
            if (!endpoint) {
                alert('Please enter an endpoint URL');
                return;
            }
            
            const method = document.getElementById('method').value;
            const requestBody = document.getElementById('request-body').value;
            
            // Collect headers
            const headers = {};
            const headerRows = document.querySelectorAll('.header-row');
            headerRows.forEach(row => {
                const keyInput = row.querySelector('.header-key');
                const valueInput = row.querySelector('.header-value');
                
                if (keyInput.value.trim()) {
                    headers[keyInput.value.trim()] = valueInput.value.trim();
                }
            });
            
            // Update UI
            const responseElement = document.getElementById('response');
            const responseInfo = document.getElementById('response-info');
            responseInfo.textContent = 'Sending request...';
            responseElement.textContent = '';
            
            // Send the request
            try {
                const start = Date.now();
                const response = await fetch(endpoint, {
                    method,
                    headers,
                    body: method !== 'GET' ? requestBody : undefined
                });
                
                const duration = Date.now() - start;
                const responseText = await response.text();
                
                // Update UI with response
                responseInfo.textContent = `Status: ${response.status} ${response.statusText} | Time: ${duration}ms`;
                responseElement.textContent = responseText;
                
                // Add to history
                addToHistory({
                    timestamp: new Date().toISOString(),
                    provider: new URL(endpoint).hostname,
                    request: JSON.stringify({
                        endpoint,
                        method,
                        headers,
                        body: requestBody
                    }, null, 2),
                    response: responseText,
                    duration_ms: duration,
                    status_code: response.status
                });
            } catch (error) {
                responseInfo.textContent = `Error: ${error.message}`;
                responseElement.textContent = error.stack;
            }
        }
        
        // Mock history functionality (in production this would use the API)
        const requestHistory = [];
        
        function addToHistory(entry) {
            requestHistory.unshift(entry);
            
            if (document.querySelector('.tab.active').getAttribute('data-tab') === 'history') {
                loadHistory();
            }
        }
        
        function loadHistory() {
            const historyContainer = document.getElementById('history-container');
            
            if (requestHistory.length === 0) {
                historyContainer.innerHTML = '<p>No request history available.</p>';
                return;
            }
            
            historyContainer.innerHTML = '';
            
            requestHistory.forEach(entry => {
                const historyItem = document.createElement('div');
                historyItem.className = 'history-item';
                
                const time = new Date(entry.timestamp).toLocaleString();
                
                historyItem.innerHTML = `
                    <div class="history-time">${time}</div>
                    <div class="history-provider">${entry.provider}</div>
                    <div>Status: ${entry.status_code} | Time: ${entry.duration_ms}ms</div>
                `;
                
                historyItem.addEventListener('click', () => {
                    // Parse the saved request
                    const request = JSON.parse(entry.request);
                    
                    // Switch to request builder tab
                    document.querySelector('.tab[data-tab="request-builder"]').click();
                    
                    // Fill the form with the saved request
                    document.getElementById('endpoint').value = request.endpoint;
                    document.getElementById('method').value = request.method;
                    document.getElementById('request-body').value = request.body;
                    
                    // Set headers
                    const headersContainer = document.getElementById('headers-container');
                    headersContainer.innerHTML = '';
                    
                    for (const [key, value] of Object.entries(request.headers)) {
                        const headerRow = document.createElement('div');
                        headerRow.className = 'header-row';
                        
                        const keyInput = document.createElement('input');
                        keyInput.className = 'header-key';
                        keyInput.value = key;
                        
                        const valueInput = document.createElement('input');
                        valueInput.className = 'header-value';
                        valueInput.value = value;
                        
                        const removeButton = document.createElement('button');
                        removeButton.className = 'remove-header';
                        removeButton.textContent = 'X';
                        removeButton.addEventListener('click', () => {
                            headersContainer.removeChild(headerRow);
                        });
                        
                        headerRow.appendChild(keyInput);
                        headerRow.appendChild(valueInput);
                        headerRow.appendChild(removeButton);
                        
                        headersContainer.appendChild(headerRow);
                    }
                    
                    // Show the saved response
                    const responseElement = document.getElementById('response');
                    const responseInfo = document.getElementById('response-info');
                    responseInfo.textContent = `Status: ${entry.status_code} | Time: ${entry.duration_ms}ms`;
                    responseElement.textContent = entry.response;
                });
                
                historyContainer.appendChild(historyItem);
            });
        }
    </script>
</body>
</html>"#;

        html.replace("TEMPLATE_OPTIONS_PLACEHOLDER", &template_options)
    }

    // Handle API requests for the request builder
    async fn handle_api_request(&self, session: &mut Session, path: &str, config: &AiRequestBuilderConfig) -> Result<bool> {
        // TODO: Implement API for fetching templates, history, etc.
        Ok(false)
    }
}

#[async_trait]
impl MiddlewarePlugin for AiRequestBuilder {
    async fn request_filter(
        &self,
        session: &mut Session,
        state: &mut RouterContext,
        plugin: &RoutePlugin,
    ) -> Result<bool> {
        let config = self.parse_config(plugin).await?;
        
        // Check if this is a request to the UI endpoint
        let request_path = session.req_header().uri.to_string();
        if request_path == config.ui_endpoint {
            return self.serve_ui(session, &config).await;
        }
        
        // Check if this is an API request
        if config.enable_api && request_path.starts_with(&format!("{}/api", config.ui_endpoint)) {
            let api_path = request_path.strip_prefix(&config.ui_endpoint).unwrap_or(&request_path);
            return self.handle_api_request(session, api_path, &config).await;
        }
        
        // Not a request for this plugin
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
        _session: &mut Session,
        _state: &mut RouterContext,
        _plugin: &RoutePlugin,
    ) -> Result<bool> {
        // No special response handling needed
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use serde_json::json;

    #[tokio::test]
    async fn test_default_config() {
        let builder = AiRequestBuilder::new();
        let config = AiRequestBuilderConfig::default();
        
        // Verify default config
        assert_eq!(config.templates.len(), 2);
        assert_eq!(config.ui_endpoint, "/ai-request-builder");
        assert!(config.enable_api);
        assert!(config.save_history);
        assert_eq!(config.max_history_entries, Some(100));
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
    }
} 