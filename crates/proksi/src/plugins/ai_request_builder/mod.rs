use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use pingora::{http::ResponseHeader, proxy::Session};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use bytes;
use http::StatusCode;
use tracing::{error, info, warn};
use html_escape;

use crate::{config::RoutePlugin, proxy_server::{https_proxy::RouterContext, HttpResponse}, plugins::core::{Plugin, PluginError, PluginStep}};

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

#[derive(Debug, Default)]
pub struct AiRequestBuilder {
    config: Arc<Mutex<AiRequestBuilderConfig>>,
    request_history: Arc<Mutex<Vec<RequestHistoryEntry>>>,
}

fn parse_plugin_config(plugin_config: Option<&RoutePlugin>) -> Result<AiRequestBuilderConfig> {
    let config_value = plugin_config
        .and_then(|p| p.config.as_ref())
        .ok_or_else(|| anyhow!("Missing plugin configuration"))?;

    let templates = if let Some(templates_val) = config_value.get("templates") {
        serde_json::from_value(templates_val.clone())
            .map_err(|e| anyhow!("Failed to parse 'templates': {}", e))?
    } else {
        AiRequestBuilderConfig::default().templates
    };

    let ui_endpoint = config_value
                        .get("ui_endpoint")
                        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| AiRequestBuilderConfig::default().ui_endpoint);

    let enable_api = config_value
                        .get("enable_api")
                        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| AiRequestBuilderConfig::default().enable_api);

    let save_history = config_value
                        .get("save_history")
                        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| AiRequestBuilderConfig::default().save_history);

    let max_history_entries = config_value
                        .get("max_history_entries")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as usize);

    Ok(AiRequestBuilderConfig {
        templates,
                        ui_endpoint,
                        enable_api,
                        save_history,
                        max_history_entries,
    })
}

impl AiRequestBuilder {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(AiRequestBuilderConfig::default())),
            request_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn add_history_entry(&self, entry: RequestHistoryEntry, config: &AiRequestBuilderConfig) {
        if !config.save_history {
            return;
        }

        let mut history = self.request_history.lock().await;
        history.push(entry);

        if let Some(max) = config.max_history_entries {
            if history.len() > max {
                history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                history.truncate(max);
            }
        }
    }

    async fn serve_ui(&self, session: &mut Session, _ctx: &mut RouterContext, config: &AiRequestBuilderConfig) -> Result<ResponseHeader> {
        let html = self.generate_ui_html(config).await;
        let content_length = html.len();

        let mut response = ResponseHeader::build(StatusCode::OK, None)?;
        response.append_header("Content-Type", "text/html; charset=utf-8")?;
        response.append_header("Content-Length", content_length.to_string())?;

        session.write_response_header(Box::new(response.clone()), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(html)), true).await?;

        Ok(response)
    }

    async fn generate_ui_html(&self, config: &AiRequestBuilderConfig) -> String {
        let mut template_options = String::new();
        for template in &config.templates {
            template_options.push_str(&format!(
                r#"<option value="{}" data-endpoint="{}" data-method="{}" data-body="{}" data-headers='{}'>{}</option>"#,
                template.name,
                template.endpoint,
                template.method,
                html_escape::encode_safe(&template.body_template),
                html_escape::encode_safe(&serde_json::to_string(&template.headers).unwrap_or_default()),
                template.name
            ));
        }

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AI Request Builder</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, "Open Sans", "Helvetica Neue", sans-serif; line-height: 1.6; padding: 20px; max-width: 900px; margin: auto; }}
        .container {{ display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }}
        .form-group {{ margin-bottom: 15px; }}
        label {{ display: block; margin-bottom: 5px; font-weight: bold; }}
        select, textarea, input[type="text"] {{ width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box; }}
        textarea {{ min-height: 150px; font-family: monospace; }}
        button {{ padding: 10px 15px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; }}
        button:hover {{ background-color: #0056b3; }}
        pre {{ background-color: #f4f4f4; padding: 10px; border-radius: 4px; white-space: pre-wrap; word-wrap: break-word; }}
        .headers-input {{ display: flex; align-items: center; margin-bottom: 5px; }}
        .headers-input input {{ flex-grow: 1; margin-right: 5px; }}
        .headers-input button {{ padding: 5px; font-size: 12px; background-color: #dc3545; }}
        #response-container {{ margin-top: 20px; }}
        #history-container {{ margin-top: 30px; border-top: 1px solid #eee; padding-top: 20px; }}
        .history-entry {{ background-color: #f9f9f9; border: 1px solid #ddd; padding: 10px; margin-bottom: 10px; border-radius: 4px; }}
        .history-entry p {{ margin: 5px 0; }}
        .history-meta {{ font-size: 0.9em; color: #555; }}
    </style>
</head>
<body>
    <h1>AI Request Builder</h1>
        <div class="container">
        <div>
            <h2>Build Request</h2>
            <div class="form-group">
                <label for="template-select">Select Provider Template:</label>
                <select id="template-select">
                    <option value="">-- Select Template --</option>
                    {template_options}
                </select>
            </div>
            <div class="form-group">
                <label for="endpoint-input">Endpoint:</label>
                <input type="text" id="endpoint-input" placeholder="e.g., api.openai.com/v1/chat/completions">
            </div>
            <div class="form-group">
                <label for="method-select">Method:</label>
                <select id="method-select">
                    <option>POST</option>
                    <option>GET</option>
                    <option>PUT</option>
                    <option>DELETE</option>
                </select>
            </div>
            <div class="form-group">
                 <label>Headers:</label>
                 <div id="headers-list">
                     </div>
                 <button type="button" id="add-header-btn">Add Header</button>
             </div>
            <div class="form-group">
                <label for="body-textarea">Request Body:</label>
                <textarea id="body-textarea" placeholder="Enter request body (e.g., JSON)"></textarea>
            </div>
            <button id="send-request-btn">Send Request</button>
        </div>
        <div>
            <h2>Response</h2>
            <div id="response-container">
                <pre id="response-output">Response will appear here...</pre>
                <p>Status: <code id="response-status">-</code></p>
                <p>Duration: <code id="response-duration">-</code> ms</p>
    </div>
             {history_section}
        </div>
    </div>

    <script>
        const templateSelect = document.getElementById('template-select');
        const endpointInput = document.getElementById('endpoint-input');
        const methodSelect = document.getElementById('method-select');
        const headersList = document.getElementById('headers-list');
        const addHeaderBtn = document.getElementById('add-header-btn');
        const bodyTextarea = document.getElementById('body-textarea');
        const sendRequestBtn = document.getElementById('send-request-btn');
        const responseOutput = document.getElementById('response-output');
        const responseStatus = document.getElementById('response-status');
        const responseDuration = document.getElementById('response-duration');
        const historyContainer = document.getElementById('history-container');

        function addHeaderInput(key = '', value = '') {{
             const div = document.createElement('div');
             div.className = 'headers-input';
             div.innerHTML = `
                 <input type="text" placeholder="Header Name" value="${{key}}">
                 <input type="text" placeholder="Header Value" value="${{value}}">
                 <button type="button" onclick="this.parentElement.remove()">Remove</button>
             `;
             headersList.appendChild(div);
         }}

        addHeaderBtn.addEventListener('click', () => addHeaderInput());

         function populateHeaders(headersObj) {{
             headersList.innerHTML = '';
             if (headersObj) {{
                 for (const [key, value] of Object.entries(headersObj)) {{
                     addHeaderInput(key, value);
                 }}
             }}
             addHeaderInput();
         }}

        templateSelect.addEventListener('change', () => {{
            const selectedOption = templateSelect.options[templateSelect.selectedIndex];
            if (!selectedOption.value) {{
                endpointInput.value = '';
                methodSelect.value = 'POST';
                bodyTextarea.value = '';
                populateHeaders({{}});
                return;
            }}
            endpointInput.value = selectedOption.dataset.endpoint || '';
            methodSelect.value = selectedOption.dataset.method || 'POST';
            bodyTextarea.value = selectedOption.dataset.body ? selectedOption.dataset.body : '';

            try {{
                const headers = JSON.parse(selectedOption.dataset.headers || '{{}}');
                populateHeaders(headers);
            }} catch (e) {{
                console.error("Error parsing headers JSON:", e);
                populateHeaders({{}});
            }}
        }});

         populateHeaders({{}});


        sendRequestBtn.addEventListener('click', async () => {{
            const providerName = templateSelect.value || 'Custom';
            const endpoint = endpointInput.value;
            const method = methodSelect.value;
            const body = bodyTextarea.value;

            const headers = {{}};
            document.querySelectorAll('.headers-input').forEach(div => {{
                 const keyInput = div.children[0];
                 const valueInput = div.children[1];
                 if (keyInput.value.trim()) {{
                     headers[keyInput.value.trim()] = valueInput.value;
                 }}
             }});


            responseOutput.textContent = 'Sending request...';
            responseStatus.textContent = '-';
            responseDuration.textContent = '-';

            try {{
                const startTime = performance.now();
                const apiUrl = `${{window.location.pathname}}/api/send`;

                const response = await fetch(apiUrl, {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json',
                    }},
                    body: JSON.stringify({{
                        provider: providerName,
                        endpoint: endpoint,
                        method: method,
                        headers: headers,
                        body: body,
                    }}),
                }});

                const endTime = performance.now();
                const duration = Math.round(endTime - startTime);

                responseStatus.textContent = response.status;
                responseDuration.textContent = duration;

                const responseData = await response.json();

                 if (response.ok) {{
                    if (responseData.response) {{
                         try {{
                             const parsedJson = JSON.parse(responseData.response);
                             responseOutput.textContent = JSON.stringify(parsedJson, null, 2);
                         }} catch (e) {{
                             responseOutput.textContent = responseData.response;
                         }}
                    }} else {{
                         responseOutput.textContent = `Request successful, but no response body received. Status: ${{responseData.status}}`;
                    }}
                    if (historyContainer) {{
                         fetchHistory();
                     }}
                }} else {{
                     responseOutput.textContent = `Error: ${{responseData.error || response.statusText}}`;
                }}

            }} catch (error) {{
                console.error('Fetch error:', error);
                responseOutput.textContent = `Network or script error: ${{error.message}}`;
                 responseStatus.textContent = 'Error';
                 responseDuration.textContent = '-';
            }}
        }});

        async function fetchHistory() {{
             if (!historyContainer) return;
             try {{
                 const historyUrl = `${{window.location.pathname}}/api/history`;
                 const response = await fetch(historyUrl);
                 if (!response.ok) {{
                     throw new Error(`Failed to fetch history: ${{response.statusText}}`);
                 }}
                 const historyData = await response.json();

                 historyContainer.innerHTML = '<h2>Request History</h2>';
                 if (historyData.length === 0) {{
                     historyContainer.innerHTML += '<p>No history saved yet.</p>';
                 }} else {{
                     historyData.forEach(entry => {{
                         const entryDiv = document.createElement('div');
                         entryDiv.className = 'history-entry';
                         entryDiv.innerHTML = `
                             <p class="history-meta"><strong>Provider:</strong> ${{entry.provider}} | <strong>Status:</strong> ${{entry.status_code}} | <strong>Duration:</strong> ${{entry.duration_ms}} ms</p>
                             <p class="history-meta"><strong>Timestamp:</strong> ${{new Date(entry.timestamp).toLocaleString()}}</p>
                             <p><strong>Request:</strong></p>
                             <pre>${{escapeHtml(entry.request)}}</pre>
                             ${{entry.response ? `<p><strong>Response:</strong></p><pre>${{escapeHtml(entry.response)}}</pre>` : '<p><strong>Response:</strong> (Not available or empty)</p>'}}
                         `;
                         historyContainer.appendChild(entryDiv);
                     }});
                 }}
             }} catch (error) {{
                 console.error('Error fetching history:', error);
                 historyContainer.innerHTML = '<h2>Request History</h2><p>Error loading history.</p>';
             }}
         }}

         function escapeHtml(unsafe) {{
            if (!unsafe) return '';
            return unsafe
                 .replace(/&/g, "&amp;")
                 .replace(/</g, "&lt;")
                 .replace(/>/g, "&gt;")
                 .replace(/"/g, "&quot;")
                 .replace(/'/g, "&#039;");
         }}


        if ({history_enabled}) {{
            document.addEventListener('DOMContentLoaded', fetchHistory);
        }}

    </script>
</body>
</html>"#,
            template_options = template_options,
            history_section = if config.save_history {
                r#"<div id="history-container"><h2>Request History</h2><p>Loading history...</p></div>"#
            } else {
                ""
            },
             history_enabled = config.save_history
        );

        html
    }

    async fn handle_api_request(&self, session: &mut Session, ctx: &mut RouterContext, path_suffix: &str, config: &AiRequestBuilderConfig) -> Result<ResponseHeader> {
        let req = session.req_header();
        let method = req.method.as_str();

        if path_suffix == "/api/send" && method == "POST" {
            let body = session.read_request_body().await?.unwrap_or_default();
            let request_data: serde_json::Value = serde_json::from_slice(&body)
                .map_err(|e| anyhow!("Failed to parse request data from UI: {}", e))?;

            let provider_name = request_data["provider"].as_str().unwrap_or("Unknown").to_string();
            let target_endpoint = request_data["endpoint"].as_str().ok_or_else(|| anyhow!("Missing 'endpoint' in API request"))?;
            let target_method = request_data["method"].as_str().ok_or_else(|| anyhow!("Missing 'method' in API request"))?;
            let target_headers_val = request_data["headers"].as_object().ok_or_else(|| anyhow!("Missing or invalid 'headers' in API request"))?;
            let target_body = request_data["body"].as_str().unwrap_or("");

            let mut target_headers = HashMap::new();
            for (k, v) in target_headers_val {
                if let Some(v_str) = v.as_str() {
                    target_headers.insert(k.clone(), v_str.to_string());
                }
            }

            info!(
                "Simulating AI request to provider: {}, Endpoint: {}, Method: {}",
                provider_name, target_endpoint, target_method
            );
            let start_time = Utc::now();

            tokio::time::sleep(tokio::time::Duration::from_millis(
                rand::random::<u64>() % 500 + 100,
            )).await;
            let simulated_status = StatusCode::OK;
            let simulated_response_body = format!(r#"{{"id": "sim_{}", "object": "text_completion", "created": {}, "model": "simulated-model", "choices": [{{"text": "This is a simulated response from {}."}}]}}"#, rand::random::<u32>(), start_time.timestamp(), provider_name);

            let end_time = Utc::now();
            let duration = end_time.signed_duration_since(start_time).num_milliseconds() as u64;
            let status_code = simulated_status.as_u16();

            let history_entry = RequestHistoryEntry {
                timestamp: start_time,
                provider: provider_name,
                request: format!("{} {}\nHeaders: {:?}\nBody: {}", target_method, target_endpoint, target_headers, target_body),
                response: Some(simulated_response_body.clone()),
                duration_ms: duration,
                status_code,
            };
            self.add_history_entry(history_entry, config).await;

            let response_payload = serde_json::json!({
                "response": simulated_response_body,
                "status": status_code,
                "duration": duration
            });
            let response_body_bytes = bytes::Bytes::from(serde_json::to_vec(&response_payload)?);

            let mut response_header = ResponseHeader::build(StatusCode::OK, None)?;
            response_header.append_header("Content-Type", "application/json")?;
            response_header.append_header("Content-Length", response_body_bytes.len().to_string())?;

            session.write_response_header(Box::new(response_header.clone()), false).await?;
            session.write_response_body(Some(response_body_bytes), true).await?;

            Ok(response_header)
        } else if path_suffix == "/api/history" && method == "GET" && config.save_history {
            let history = self.request_history.lock().await;
            let history_json = serde_json::to_vec(&*history)?;
            let history_bytes = bytes::Bytes::from(history_json);

            let mut response_header = ResponseHeader::build(StatusCode::OK, None)?;
            response_header.append_header("Content-Type", "application/json")?;
            response_header.append_header("Content-Length", history_bytes.len().to_string())?;

            session.write_response_header(Box::new(response_header.clone()), false).await?;
            session.write_response_body(Some(history_bytes), true).await?;

            Ok(response_header)
        } else {
            warn!("Unhandled API request: {} {}", method, path_suffix);
            let mut response_header = ResponseHeader::build(StatusCode::NOT_FOUND, None)?;
            response_header.append_header("Content-Type", "application/json")?;
            let body = r#"{"error": "API endpoint not found"}"#.as_bytes();
            response_header.append_header("Content-Length", body.len().to_string())?;

            session.write_response_header(Box::new(response_header.clone()), false).await?;
            session.write_response_body(Some(bytes::Bytes::from_static(body)), true).await?;

            Ok(response_header)
        }
    }
}

#[async_trait]
impl Plugin for AiRequestBuilder {
    fn name(&self) -> &'static str {
        "AiRequestBuilder"
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        info!("AiRequestBuilder plugin started.");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("AiRequestBuilder plugin stopped.");
        Ok(())
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        if step != PluginStep::Request {
            return Ok((false, None));
        }
        
        // Get a snapshot of the config to avoid locking issues
        let config = self.config.lock().await.clone();
        let req_path = session.req_header().uri.path().to_string();  // Clone to avoid immutable borrow issues
        
        if config.ui_endpoint.is_empty() || !req_path.starts_with(&config.ui_endpoint) {
            return Ok((false, None));
        }
        
        info!("Path: {}, UI Endpoint: {}", req_path, config.ui_endpoint);
        
        if req_path == config.ui_endpoint || req_path == format!("{}/", config.ui_endpoint) {
             match self.serve_ui(session, ctx, &config).await {
                 Ok(response_header) => Ok((true, Some(HttpResponse::new(
                     response_header.status, 
                     response_header.headers.clone(), 
                     bytes::Bytes::new()
                 )))),
                 Err(e) => {
                     error!("Error serving AiRequestBuilder UI: {}", e);
                     let err_resp = ResponseHeader::build(StatusCode::INTERNAL_SERVER_ERROR, None)?;
                     session.write_response_header(Box::new(err_resp.clone()), true).await?;
                     Ok((true, Some(HttpResponse::new(
                         err_resp.status,
                         err_resp.headers.clone(),
                         bytes::Bytes::new()
                     ))))
                 }
             }
        } else if config.enable_api && req_path.starts_with(&format!("{}/api/", config.ui_endpoint)) {
             let path_suffix = req_path.strip_prefix(&config.ui_endpoint).unwrap_or(&req_path);
             info!("Handling AiRequestBuilder API request for path suffix: {}", path_suffix);
              match self.handle_api_request(session, ctx, path_suffix, &config).await {
                  Ok(response_header) => Ok((true, Some(HttpResponse::new(
                      response_header.status,
                      response_header.headers.clone(),
                      bytes::Bytes::new()
                  )))),
                  Err(e) => {
                     error!("Error handling AiRequestBuilder API request: {}", e);
                     let mut err_resp = ResponseHeader::build(StatusCode::INTERNAL_SERVER_ERROR, None)?;
                     let error_payload = serde_json::json!({ "error": format!("API Error: {}", e) });
                     let err_body = serde_json::to_vec(&error_payload)?;
                     err_resp.append_header("Content-Type", "application/json")?;
                     err_resp.append_header("Content-Length", err_body.len().to_string())?;
                     session.write_response_header(Box::new(err_resp.clone()), false).await?;
                     session.write_response_body(Some(bytes::Bytes::from(err_body)), true).await?;
                     Ok((true, Some(HttpResponse::new(
                         err_resp.status,
                         err_resp.headers.clone(),
                         bytes::Bytes::new()
                     ))))
                 }
             }
        } else {
            Ok((false, None))
        }
    }

    async fn handle_response(
        &self,
        _step: PluginStep,
        _session: &mut Session,
        _ctx: &mut RouterContext,
        _upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        Ok(false)
    }
} 