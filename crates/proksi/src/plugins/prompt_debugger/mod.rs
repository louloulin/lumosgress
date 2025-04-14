use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use http::StatusCode;
use pingora::{http::{RequestHeader, ResponseHeader}, proxy::Session};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use bytes;
use serde_json::Value;
use tracing::warn;

use crate::{config::RoutePlugin, proxy_server::https_proxy::RouterContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugRule {
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub suggestion: String,
    pub severity: RuleSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDebuggerConfig {
    pub ui_endpoint: String,
    pub enable_api: bool,
    pub max_history_entries: Option<usize>,
    pub rules: Vec<DebugRule>,
    pub custom_rules_endpoint: Option<String>,
    pub intercept_requests: bool,
}

impl Default for PromptDebuggerConfig {
    fn default() -> Self {
        Self {
            ui_endpoint: "/prompt-debugger".to_string(),
            enable_api: true,
            max_history_entries: Some(100),
            rules: vec![
                DebugRule {
                    name: "System Role First".to_string(),
                    description: "Check if system message appears before user messages".to_string(),
                    pattern: r#""role":\s*"system""#.to_string(),
                    suggestion: "Place system messages at the beginning of your conversation to establish context.".to_string(),
                    severity: RuleSeverity::Warning,
                },
                DebugRule {
                    name: "Clear Instructions".to_string(),
                    description: "Check if instructions are clear and specific".to_string(),
                    pattern: r#"(what|how|why|who|when|where|please|could you|can you)"#.to_string(),
                    suggestion: "Your prompt appears to be clear and question-based, which is good for getting specific responses.".to_string(),
                    severity: RuleSeverity::Info,
                },
                DebugRule {
                    name: "Long Prompt".to_string(),
                    description: "Check if prompt is excessively long".to_string(),
                    pattern: r#".{1000,}"#.to_string(),
                    suggestion: "Your prompt is quite long. Consider breaking it into smaller, more focused prompts for better responses.".to_string(),
                    severity: RuleSeverity::Warning,
                },
                DebugRule {
                    name: "Vague Language".to_string(),
                    description: "Check for vague or ambiguous language".to_string(),
                    pattern: r#"(stuff|things|something|anything|good|nice|better|etc\.)"#.to_string(),
                    suggestion: "Your prompt contains vague terms. Try to be more specific for better results.".to_string(),
                    severity: RuleSeverity::Warning,
                },
                DebugRule {
                    name: "Few-Shot Example".to_string(),
                    description: "Check if few-shot examples are provided".to_string(),
                    pattern: r#"(example|examples|for instance|e\.g\.)"#.to_string(),
                    suggestion: "Good use of examples! Examples help models understand the desired output format.".to_string(),
                    severity: RuleSeverity::Info,
                },
            ],
            custom_rules_endpoint: None,
            intercept_requests: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAnalysisResult {
    pub timestamp: DateTime<Utc>,
    pub prompt: String,
    pub results: Vec<RuleResult>,
    pub suggestions: Vec<String>,
    pub improved_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_name: String,
    pub matches: bool,
    pub details: String,
    pub severity: RuleSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub original_prompt: String,
    pub analysis_result: PromptAnalysisResult,
    pub model: Option<String>,
    pub response: Option<String>,
}

pub struct PromptDebugger {
    config: Arc<Mutex<HashMap<String, PromptDebuggerConfig>>>,
    history: Arc<Mutex<Vec<PromptHistoryEntry>>>,
}

impl PromptDebugger {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Parse configuration from the plugin
    async fn parse_config(&self, plugin: &RoutePlugin) -> Result<PromptDebuggerConfig> {
        if let Some(config) = &plugin.config {
            if let Some(config_name) = config.get("config_name") {
                if let Some(config_name) = config_name.as_str() {
                    let configs = self.config.lock().await;
                    if let Some(config) = configs.get(config_name) {
                        return Ok(config.clone());
                    }
                }
            }

            // Parse custom UI endpoint
            let ui_endpoint = config
                .get("ui_endpoint")
                .and_then(|v| v.as_str())
                .unwrap_or("/prompt-debugger")
                .to_string();

            // Parse enable_api
            let enable_api = config
                .get("enable_api")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            // Parse max_history_entries
            let max_history_entries = config
                .get("max_history_entries")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);

            // Parse intercept_requests
            let intercept_requests = config
                .get("intercept_requests")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Parse custom_rules_endpoint
            let custom_rules_endpoint = config
                .get("custom_rules_endpoint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Parse custom rules if provided
            let rules = if let Some(rules_array) = config.get("rules").and_then(|v| v.as_array()) {
                let mut debug_rules = Vec::new();

                for rule in rules_array {
                    if let Some(rule_obj) = rule.as_object() {
                        if let (
                            Some(name),
                            Some(description),
                            Some(pattern),
                            Some(suggestion),
                            Some(severity),
                        ) = (
                            rule_obj.get("name").and_then(|v| v.as_str()),
                            rule_obj.get("description").and_then(|v| v.as_str()),
                            rule_obj.get("pattern").and_then(|v| v.as_str()),
                            rule_obj.get("suggestion").and_then(|v| v.as_str()),
                            rule_obj.get("severity").and_then(|v| v.as_str()),
                        ) {
                            let severity = match severity.to_lowercase().as_str() {
                                "info" => RuleSeverity::Info,
                                "warning" => RuleSeverity::Warning,
                                "error" => RuleSeverity::Error,
                                _ => RuleSeverity::Info,
                            };

                            debug_rules.push(DebugRule {
                                name: name.to_string(),
                                description: description.to_string(),
                                pattern: pattern.to_string(),
                                suggestion: suggestion.to_string(),
                                severity,
                            });
                        }
                    }
                }

                debug_rules
            } else {
                PromptDebuggerConfig::default().rules
            };

            return Ok(PromptDebuggerConfig {
                ui_endpoint,
                enable_api,
                max_history_entries,
                rules,
                custom_rules_endpoint,
                intercept_requests,
            });
        }

        // Return default configuration if no specific config provided
        Ok(PromptDebuggerConfig::default())
    }

    // Analyze a prompt using the debug rules
    fn analyze_prompt(&self, prompt: &str, config: &PromptDebuggerConfig) -> PromptAnalysisResult {
        let mut results = Vec::new();
        let mut suggestions = Vec::new();

        for rule in &config.rules {
            let re = regex::Regex::new(&rule.pattern).unwrap_or_else(|_| {
                regex::Regex::new(r"$.").unwrap() // Dummy regex that won't match anything
            });

            let matches = re.is_match(prompt);
            let details = if matches {
                match rule.severity {
                    RuleSeverity::Info => rule.suggestion.clone(),
                    RuleSeverity::Warning | RuleSeverity::Error => {
                        suggestions.push(rule.suggestion.clone());
                        rule.suggestion.clone()
                    }
                }
            } else {
                "No match found".to_string()
            };

            results.push(RuleResult {
                rule_name: rule.name.clone(),
                matches,
                details,
                severity: rule.severity,
            });
        }

        // Create an improved prompt suggestion (in a real implementation this could be more sophisticated)
        let improved_prompt = if !suggestions.is_empty() {
            Some(prompt.to_string())
        } else {
            None
        };

        PromptAnalysisResult {
            timestamp: Utc::now(),
            prompt: prompt.to_string(),
            results,
            suggestions,
            improved_prompt,
        }
    }

    // Extract prompt from request body
    fn extract_prompt_from_request(&self, body: &str) -> Option<String> {
        // Try to parse as JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
            // Check for different LLM provider formats
            
            // OpenAI format (messages array)
            if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
                for msg in messages {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                            if role == "user" {
                                return Some(content.to_string());
                            }
                        }
                    }
                }
            } 
            
            // Anthropic format
            else if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
                for msg in messages {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                            if role == "user" {
                                return Some(content.to_string());
                            }
                        }
                    }
                }
            } 
            
            // Google format
            else if let Some(contents) = json.get("contents").and_then(|c| c.as_array()) {
                for content in contents {
                    if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                        for part in parts {
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                return Some(text.to_string());
                            }
                        }
                    }
                }
            }
            
            // Generic prompt formats
            else if let Some(prompt) = json.get("prompt").and_then(|p| p.as_str()) {
                return Some(prompt.to_string());
            } else if let Some(input) = json.get("input").and_then(|i| i.as_str()) {
                return Some(input.to_string());
            } else if let Some(query) = json.get("query").and_then(|q| q.as_str()) {
                return Some(query.to_string());
            }
        }
        
        // Fallback: just return the body if it's not too long
        if body.len() < 10000 {
            return Some(body.to_string());
        }
        
        None
    }

    // Add entry to history
    async fn add_history_entry(
        &self,
        entry: PromptHistoryEntry,
        config: &PromptDebuggerConfig,
    ) -> Result<()> {
        let mut history = self.history.lock().await;
        history.push(entry);

        // Prune history if needed
        if let Some(max) = config.max_history_entries {
            if history.len() > max {
                history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                history.truncate(max);
            }
        }

        Ok(())
    }

    // Serve the UI
    async fn serve_ui(&self, session: &mut Session, config: &PromptDebuggerConfig) -> Result<bool> {
        let html = self.generate_ui_html(config).await;
        let content_length = html.len();

        // Write HTTP response
        let mut response = ResponseHeader::build(StatusCode::OK, None)?;
        response.append_header("Content-Type", "text/html; charset=utf-8")?;
        response.append_header("Content-Length", content_length.to_string())?;

        session.write_response_header(Box::new(response), false).await?;
        session.write_response_body(Some(bytes::Bytes::from(html)), true).await?;

        Ok(true)
    }

    // Generate the HTML for the UI
    async fn generate_ui_html(&self, config: &PromptDebuggerConfig) -> String {
        let rules_json = serde_json::to_string(&config.rules).unwrap_or_else(|_| "[]".to_string());
        
        // Use a raw string (r#) to avoid escaping issues
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Prompt Debugger</title>
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
            min-height: 200px;
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
        .rule-result {
            margin-bottom: 10px;
            padding: 10px;
            border-radius: 4px;
        }
        .rule-info {
            background-color: #e3f2fd;
            border-left: 4px solid #2196F3;
        }
        .rule-warning {
            background-color: #fff8e1;
            border-left: 4px solid #ffc107;
        }
        .rule-error {
            background-color: #ffebee;
            border-left: 4px solid #f44336;
        }
        .rule-title {
            font-weight: bold;
            margin-bottom: 5px;
        }
        .suggestions {
            margin-top: 20px;
            padding: 10px;
            background-color: #e8f5e9;
            border-left: 4px solid #4caf50;
        }
        .improved-prompt {
            margin-top: 20px;
            padding: 10px;
            background-color: #f5f5f5;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-family: monospace;
            white-space: pre-wrap;
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
        .rules-container {
            margin-top: 20px;
        }
        .rule-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 10px;
        }
        .rule-header h2 {
            margin: 0;
        }
        .rule-card {
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 10px;
            margin-bottom: 10px;
        }
        .rule-card h3 {
            margin-top: 0;
        }
        .severity-info {
            color: #2196F3;
        }
        .severity-warning {
            color: #ffc107;
        }
        .severity-error {
            color: #f44336;
        }
    </style>
</head>
<body>
    <h1>Prompt Debugger</h1>
    
    <div class="tabs">
        <div class="tab active" data-tab="analyzer">Prompt Analyzer</div>
        <div class="tab" data-tab="history">Analysis History</div>
        <div class="tab" data-tab="rules">Debug Rules</div>
    </div>
    
    <div id="analyzer" class="tab-content active">
        <div class="container">
            <div class="left-panel">
                <h2>Your Prompt</h2>
                <textarea id="prompt-input" placeholder="Enter your prompt here..."></textarea>
                <button id="analyze-button">Analyze Prompt</button>
            </div>
            
            <div class="right-panel">
                <h2>Analysis Results</h2>
                <div id="analysis-results"></div>
                
                <div id="suggestions-container" class="suggestions" style="display: none;">
                    <h3>Suggestions</h3>
                    <ul id="suggestions-list"></ul>
                </div>
                
                <div id="improved-prompt-container" style="display: none;">
                    <h3>Improved Prompt</h3>
                    <div id="improved-prompt" class="improved-prompt"></div>
                </div>
            </div>
        </div>
    </div>
    
    <div id="history" class="tab-content">
        <h2>Analysis History</h2>
        <div id="history-container">
            <p>No analysis history available.</p>
        </div>
    </div>
    
    <div id="rules" class="tab-content">
        <div class="rule-header">
            <h2>Debug Rules</h2>
        </div>
        <div id="rules-container">
            <!-- Rules will be populated here -->
        </div>
    </div>

    <script>
        // Debug rules data from server
        const debugRules = JSON.parse(`RULES_JSON_PLACEHOLDER`);
        
        // Initialize with rules data
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
                    
                    if (targetTab === 'rules') {
                        populateRules();
                    } else if (targetTab === 'history') {
                        loadHistory();
                    }
                });
            });
            
            // Set up analyze button
            document.getElementById('analyze-button').addEventListener('click', analyzePrompt);
            
            // Populate rules tab initially
            populateRules();
        });
        
        function populateRules() {
            const rulesContainer = document.getElementById('rules-container');
            rulesContainer.innerHTML = '';
            
            debugRules.forEach(rule => {
                const ruleCard = document.createElement('div');
                ruleCard.className = 'rule-card';
                
                const severityClass = `severity-${rule.severity.toLowerCase()}`;
                
                ruleCard.innerHTML = `
                    <h3>${rule.name} <span class="${severityClass}">[${rule.severity}]</span></h3>
                    <p><strong>Description:</strong> ${rule.description}</p>
                    <p><strong>Pattern:</strong> <code>${rule.pattern}</code></p>
                    <p><strong>Suggestion:</strong> ${rule.suggestion}</p>
                `;
                
                rulesContainer.appendChild(ruleCard);
            });
        }
        
        function analyzePrompt() {
            const promptText = document.getElementById('prompt-input').value.trim();
            
            if (!promptText) {
                alert('Please enter a prompt to analyze');
                return;
            }
            
            // Perform analysis (this would usually be a server call)
            const results = [];
            const suggestions = [];
            
            debugRules.forEach(rule => {
                try {
                    const regex = new RegExp(rule.pattern);
                    const matches = regex.test(promptText);
                    
                    let details = 'No match found';
                    if (matches) {
                        details = rule.suggestion;
                        if (rule.severity !== 'Info') {
                            suggestions.push(rule.suggestion);
                        }
                    }
                    
                    results.push({
                        ruleName: rule.name,
                        matches,
                        details,
                        severity: rule.severity
                    });
                } catch (error) {
                    console.error(`Error with regex pattern ${rule.pattern}:`, error);
                    results.push({
                        ruleName: rule.name,
                        matches: false,
                        details: `Error evaluating rule: ${error.message}`,
                        severity: 'Error'
                    });
                }
            });
            
            displayResults(results, suggestions, promptText);
            
            // Add to history
            addToHistory({
                timestamp: new Date().toISOString(),
                originalPrompt: promptText,
                analysisResult: {
                    timestamp: new Date().toISOString(),
                    prompt: promptText,
                    results,
                    suggestions,
                    improvedPrompt: suggestions.length > 0 ? promptText : null
                }
            });
        }
        
        function displayResults(results, suggestions, originalPrompt) {
            const resultsContainer = document.getElementById('analysis-results');
            resultsContainer.innerHTML = '';
            
            if (results.length === 0) {
                resultsContainer.innerHTML = '<p>No results to display</p>';
                return;
            }
            
            results.forEach(result => {
                const resultEl = document.createElement('div');
                resultEl.className = `rule-result rule-${result.severity.toLowerCase()}`;
                
                resultEl.innerHTML = `
                    <div class="rule-title">${result.ruleName} [${result.severity}]</div>
                    <div>${result.matches ? '✓ Match found' : '✗ No match'}</div>
                    <div>${result.details}</div>
                `;
                
                resultsContainer.appendChild(resultEl);
            });
            
            // Show suggestions if any
            const suggestionsContainer = document.getElementById('suggestions-container');
            const suggestionsList = document.getElementById('suggestions-list');
            
            if (suggestions.length > 0) {
                suggestionsList.innerHTML = '';
                suggestions.forEach(suggestion => {
                    const li = document.createElement('li');
                    li.textContent = suggestion;
                    suggestionsList.appendChild(li);
                });
                suggestionsContainer.style.display = 'block';
            } else {
                suggestionsContainer.style.display = 'none';
            }
            
            // Show improved prompt if there are suggestions
            const improvedPromptContainer = document.getElementById('improved-prompt-container');
            const improvedPrompt = document.getElementById('improved-prompt');
            
            if (suggestions.length > 0) {
                improvedPrompt.textContent = originalPrompt;
                improvedPromptContainer.style.display = 'block';
            } else {
                improvedPromptContainer.style.display = 'none';
            }
        }
        
        // Mock history functionality (in production this would use the API)
        const promptHistory = [];
        
        function addToHistory(entry) {
            promptHistory.unshift(entry);
            
            if (document.querySelector('.tab.active').getAttribute('data-tab') === 'history') {
                loadHistory();
            }
        }
        
        function loadHistory() {
            const historyContainer = document.getElementById('history-container');
            
            if (promptHistory.length === 0) {
                historyContainer.innerHTML = '<p>No analysis history available.</p>';
                return;
            }
            
            historyContainer.innerHTML = '';
            
            promptHistory.forEach(entry => {
                const historyItem = document.createElement('div');
                historyItem.className = 'history-item';
                
                const time = new Date(entry.timestamp).toLocaleString();
                const promptPreview = entry.originalPrompt.length > 50 
                    ? entry.originalPrompt.substring(0, 50) + '...' 
                    : entry.originalPrompt;
                
                const issueCount = entry.analysisResult.results.filter(r => r.matches && r.severity !== 'Info').length;
                
                historyItem.innerHTML = `
                    <div class="history-time">${time}</div>
                    <div>${promptPreview}</div>
                    <div>Issues found: ${issueCount}</div>
                `;
                
                historyItem.addEventListener('click', () => {
                    // Switch to analyzer tab
                    document.querySelector('.tab[data-tab="analyzer"]').click();
                    
                    // Set the prompt
                    document.getElementById('prompt-input').value = entry.originalPrompt;
                    
                    // Show the results
                    displayResults(
                        entry.analysisResult.results,
                        entry.analysisResult.suggestions,
                        entry.originalPrompt
                    );
                });
                
                historyContainer.appendChild(historyItem);
            });
        }
    </script>
</body>
</html>"#.replace("RULES_JSON_PLACEHOLDER", &rules_json)
    }

    // Handle API requests
    async fn handle_api_request(&self, session: &mut Session, path: &str, config: &PromptDebuggerConfig) -> Result<bool> {
        match path {
            "/api/analyze" => {
                // Read request body to analyze
                if let Some(body) = session.read_request_body().await.ok().flatten() {
                    if let Some(prompt) = self.extract_prompt_from_request(&String::from_utf8_lossy(&body)) {
                        // Analyze the prompt
                        let analysis = self.analyze_prompt(&prompt, config);
                        
                        // Create response
                        let json = serde_json::to_string(&analysis)?;
                        let content_length = json.len();
                        
                        let mut response = ResponseHeader::build(StatusCode::OK, None)?;
                        response.append_header("Content-Type", "application/json")?;
                        response.append_header("Content-Length", content_length.to_string())?;
                        
                        session.write_response_header(Box::new(response), false).await?;
                        session.write_response_body(Some(bytes::Bytes::from(json)), true).await?;
                        
                        return Ok(true);
                    }
                }
                
                // Return 400 if we couldn't extract a prompt
                let mut response = ResponseHeader::build(StatusCode::BAD_REQUEST, None)?;
                let error_msg = "Could not extract prompt from request";
                response.append_header("Content-Type", "text/plain")?;
                response.append_header("Content-Length", error_msg.len().to_string())?;
                
                session.write_response_header(Box::new(response), false).await?;
                session.write_response_body(Some(bytes::Bytes::from(error_msg)), true).await?;
                
                Ok(true)
            },
            "/api/rules" => {
                // Return the configured rules
                let json = serde_json::to_string(&config.rules)?;
                let content_length = json.len();
                
                let mut response = ResponseHeader::build(StatusCode::OK, None)?;
                response.append_header("Content-Type", "application/json")?;
                response.append_header("Content-Length", content_length.to_string())?;
                
                session.write_response_header(Box::new(response), false).await?;
                session.write_response_body(Some(bytes::Bytes::from(json)), true).await?;
                
                Ok(true)
            },
            "/api/history" => {
                // Return the analysis history
                let history = self.history.lock().await;
                let json = serde_json::to_string(&*history)?;
                let content_length = json.len();
                
                let mut response = ResponseHeader::build(StatusCode::OK, None)?;
                response.append_header("Content-Type", "application/json")?;
                response.append_header("Content-Length", content_length.to_string())?;
                
                session.write_response_header(Box::new(response), false).await?;
                session.write_response_body(Some(bytes::Bytes::from(json)), true).await?;
                
                Ok(true)
            },
            _ => Ok(false),
        }
    }
}

/* // Commented out outdated implementation
#[async_trait]
impl MiddlewarePlugin for PromptDebugger {
    // ... implementation ...
}
*/ 