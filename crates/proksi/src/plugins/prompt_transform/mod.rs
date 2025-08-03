use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use http::HeaderValue;
use pingora::{
    http::ResponseHeader,
    proxy::Session,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

// Import new Plugin trait and related types
use crate::plugins::core::{Plugin, PluginError, PluginStep, PluginMetadata, PluginType};
use crate::proxy_server::HttpResponse;

use crate::proxy_server::https_proxy::RouterContext;

// 提示转换类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    // 添加系统消息
    AddSystemMessage,
    // 拼接额外上下文
    AddContext,
    // 添加安全检查
    AddSafetyCheck,
    // 提示模板
    ApplyTemplate,
    // 格式化提示
    FormatPrompt,
    // 提取关键词
    ExtractKeywords,
    // 内容增强
    EnhanceContent,
    // 语言翻译
    TranslatePrompt,
    // 提示分割
    SplitPrompt,
    // RAG增强
    RAGEnhancement,
    // 自定义转换
    Custom,
}

impl From<&str> for TransformationType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "system" | "system_message" => TransformationType::AddSystemMessage,
            "context" | "add_context" => TransformationType::AddContext,
            "safety" | "safety_check" => TransformationType::AddSafetyCheck,
            "template" | "apply_template" => TransformationType::ApplyTemplate,
            "format" | "format_prompt" => TransformationType::FormatPrompt,
            "keywords" | "extract_keywords" => TransformationType::ExtractKeywords,
            "enhance" | "enhance_content" => TransformationType::EnhanceContent,
            "translate" | "translate_prompt" => TransformationType::TranslatePrompt,
            "split" | "split_prompt" => TransformationType::SplitPrompt,
            "rag" | "rag_enhancement" => TransformationType::RAGEnhancement,
            _ => TransformationType::Custom,
        }
    }
}

// 提示转换配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptTransformConfig {
    pub transformations: Vec<PromptTransformation>,
}

// 单个转换配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTransformation {
    pub transform_type: TransformationType,
    pub provider: Option<String>, // 适用的提供商，None表示全部
    pub content: String,          // 转换内容
    pub template: Option<String>, // 模板名称
    pub target_lang: Option<String>,      // 目标语言（用于翻译）
    pub format_style: Option<String>,     // 格式化样式
    pub max_tokens: Option<usize>,        // 最大Token数（用于分割）
    pub enhancement_level: Option<String>, // 增强级别（基础、中级、高级）
    pub rag_source: Option<String>,       // RAG数据源
    pub custom_params: Option<Value>,     // 自定义参数（JSON格式）
}

// Modify PromptTransformer struct
#[derive(Debug, Clone)]
pub struct PromptTransformer {
    pub config: PromptTransformConfig,
    pub templates: HashMap<String, String>,
}

impl PromptTransformer {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(
            "general_instruction".to_string(),
            "You are an AI assistant that provides helpful, accurate, and safe information. {{prompt}}".to_string(),
        );
        templates.insert(
            "safety_check".to_string(),
            "Ensure your response is ethical and does not contain harmful content. {{prompt}}".to_string(),
        );

        Self {
            config: PromptTransformConfig::default(),
            templates,
        }
    }

    pub fn with_config(config: PromptTransformConfig, templates: Option<HashMap<String, String>>) -> Self {
        let final_templates = templates.unwrap_or_else(|| {
            let mut default_templates = HashMap::new();
            default_templates.insert(
                "general_instruction".to_string(),
                "You are an AI assistant that provides helpful, accurate, and safe information. {{prompt}}".to_string(),
            );
            default_templates.insert(
                "safety_check".to_string(),
                "Ensure your response is ethical and does not contain harmful content. {{prompt}}".to_string(),
            );
            default_templates
        });
        
        Self { 
            config,
            templates: final_templates,
        }
    }

    // 添加系统消息
    pub fn add_system_message(&self, json_body: &mut Value, content: &str) -> bool {
        // 对OpenAI格式的处理
        if let Some(messages) = json_body.get_mut("messages").and_then(|m| m.as_array_mut()) {
            // 检查是否已经有系统消息
            let has_system = messages.iter().any(|m| {
                m.get("role").and_then(|r| r.as_str()) == Some("system")
            });

            if !has_system {
                // 在最前面添加系统消息
                messages.insert(0, json!({
                    "role": "system",
                    "content": content
                }));
                return true;
            }
        }
        // 对Anthropic格式的处理
        else if let Some(prompt) = json_body.get_mut("prompt").and_then(|p| p.as_str()) {
            // 检查是否已经有Human:开头
            if prompt.trim().starts_with("Human:") {
                let system_content = format!("System: {}\n\n{}", content, prompt);
                json_body["prompt"] = system_content.into();
                return true;
            }
        }
        
        false
    }

    // 添加上下文
    fn add_context(&self, json_body: &mut Value, content: &str) -> bool {
        // 对OpenAI格式的处理
        if let Some(messages) = json_body.get_mut("messages").and_then(|m| m.as_array_mut()) {
            for message in messages.iter_mut() {
                if message.get("role").and_then(|r| r.as_str()) == Some("user") {
                    if let Some(msg_content) = message.get_mut("content").and_then(|c| c.as_str()) {
                        let new_content = format!("{}\n\nAdditional context: {}", msg_content, content);
                        message["content"] = new_content.into();
                        return true;
                    }
                }
            }
        }
        // 对Anthropic格式的处理
        else if let Some(prompt) = json_body.get_mut("prompt").and_then(|p| p.as_str()) {
            if let Some(human_idx) = prompt.find("Human:") {
                let prefix = &prompt[0..human_idx];
                let human_part = &prompt[human_idx..];
                
                // 在Human部分后添加上下文
                let new_prompt = format!("{}{}Additional context: {}", prefix, human_part, content);
                json_body["prompt"] = new_prompt.into();
                return true;
            }
        }
        
        false
    }

    // 添加安全检查
    fn add_safety_check(&self, json_body: &mut Value, content: &str) -> bool {
        // 对OpenAI格式的处理
        if let Some(messages) = json_body.get_mut("messages").and_then(|m| m.as_array_mut()) {
            // 找到系统消息或添加一个
            let has_system = messages.iter().any(|m| {
                m.get("role").and_then(|r| r.as_str()) == Some("system")
            });

            if has_system {
                for message in messages.iter_mut() {
                    if message.get("role").and_then(|r| r.as_str()) == Some("system") {
                        if let Some(msg_content) = message.get_mut("content").and_then(|c| c.as_str()) {
                            let new_content = format!("{}\n\nSafety instructions: {}", msg_content, content);
                            message["content"] = new_content.into();
                            return true;
                        }
                    }
                }
            } else {
                // 添加新的系统消息
                messages.insert(0, json!({
                    "role": "system",
                    "content": format!("Safety instructions: {}", content)
                }));
                return true;
            }
        }
        // 对Anthropic格式的处理
        else if let Some(prompt) = json_body.get_mut("prompt").and_then(|p| p.as_str()) {
            if let Some(system_idx) = prompt.find("System:") {
                // 已有系统指令，附加安全指令
                let prefix = &prompt[0..system_idx + 7]; // "System:"后
                let rest = &prompt[system_idx + 7..];
                
                let new_prompt = format!("{} Safety instructions: {}.{}", prefix, content, rest);
                json_body["prompt"] = new_prompt.into();
                return true;
            } else if let Some(human_idx) = prompt.find("Human:") {
                // 没有系统指令，在Human前添加
                let prefix = &prompt[0..human_idx];
                let human_part = &prompt[human_idx..];
                
                let new_prompt = format!("{}System: Safety instructions: {}.\n\n{}", prefix, content, human_part);
                json_body["prompt"] = new_prompt.into();
                return true;
            }
        }
        
        false
    }

    fn apply_template(&self, json_body: &mut Value, template_name: &str) -> bool {
        if let Some(template) = self.templates.get(template_name) {
            // 对OpenAI格式的处理
            if let Some(messages) = json_body.get_mut("messages").and_then(|m| m.as_array_mut()) {
                // 只处理user消息
                for message in messages.iter_mut() {
                    if message.get("role").and_then(|r| r.as_str()) == Some("user") {
                        if let Some(msg_content) = message.get_mut("content").and_then(|c| c.as_str()) {
                            let new_content = template.replace("{{prompt}}", msg_content);
                            message["content"] = new_content.into();
                            return true;
                        }
                    }
                }
            }
            // 对Anthropic格式的处理
            else if let Some(prompt) = json_body.get_mut("prompt").and_then(|p| p.as_str()) {
                if let Some(human_idx) = prompt.find("Human:") {
                    let human_content = &prompt[human_idx + 6..]; // "Human:"后
                    
                    let new_prompt = format!("Human: {}", template.replace("{{prompt}}", human_content));
                    json_body["prompt"] = new_prompt.into();
                    return true;
                }
            }
        }
        
        false
    }

    // 格式化提示
    fn format_prompt(&self, json_body: &mut Value, style: &str, instructions: &str) -> bool {
        // 获取原始提示内容
        let original_prompt = self.extract_user_prompt(json_body);
        if original_prompt.is_none() {
            return false;
        }
        
        let prompt = original_prompt.unwrap();
        let formatted_prompt = match style.to_lowercase().as_str() {
            "bullet" => {
                // 将文本转换为项目符号列表
                let bullets: Vec<String> = prompt
                    .split('.')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| format!("• {}", s.trim()))
                    .collect();
                bullets.join("\n")
            }
            "numbered" => {
                // 将文本转换为编号列表
                let numbered: Vec<String> = prompt
                    .split('.')
                    .filter(|s| !s.trim().is_empty())
                    .enumerate()
                    .map(|(i, s)| format!("{}. {}", i + 1, s.trim()))
                    .collect();
                numbered.join("\n")
            }
            "structured" => {
                // 添加标题和结构
                format!("# {}\n\n{}", instructions, prompt)
            }
            "concise" => {
                // 简明格式（去除冗余词语）
                prompt.replace("please ", "")
                    .replace("could you ", "")
                    .replace("I would like to ", "")
                    .replace("I want to ", "")
            }
            _ => {
                // 默认格式化方式
                format!("{}\n\n{}", instructions, prompt)
            }
        };
        
        // 更新提示内容
        self.update_user_prompt(json_body, &formatted_prompt)
    }

    // 提取关键词
    fn extract_keywords(&self, json_body: &mut Value, instructions: &str) -> bool {
        // 获取原始提示内容
        let original_prompt = self.extract_user_prompt(json_body);
        if original_prompt.is_none() {
            return false;
        }
        
        let prompt = original_prompt.unwrap();
        
        // 基本的关键词提取逻辑（在实际实现中可使用更复杂的NLP方法）
        let stop_words = vec!["a", "an", "the", "in", "on", "at", "to", "for", "with", "by", "about", "like"];
        let keywords: Vec<String> = prompt
            .split_whitespace()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| s.len() > 3 && !stop_words.contains(&s.as_str()))
            .take(10) // 限制关键词数量
            .collect();
        
        // 构建增强提示
        let enhanced_prompt = format!(
            "{}\n\nKeywords: {}\n\nOriginal request: {}",
            instructions,
            keywords.join(", "),
            prompt
        );
        
        // 更新提示内容
        self.update_user_prompt(json_body, &enhanced_prompt)
    }

    // 增强内容
    fn enhance_content(&self, json_body: &mut Value, level: &str, instructions: &str) -> bool {
        // 获取原始提示内容
        let original_prompt = self.extract_user_prompt(json_body);
        if original_prompt.is_none() {
            return false;
        }
        
        let prompt = original_prompt.unwrap();
        
        // 根据不同级别应用不同的增强
        let enhanced_prompt = match level {
            "basic" => {
                format!("{}\n\n{}", prompt, instructions)
            }
            "standard" => {
                format!(
                    "Please provide a detailed and comprehensive response to the following request:\n\n{}\n\n{}",
                    prompt, instructions
                )
            }
            "advanced" => {
                format!(
                    "As an expert in this subject, please provide an in-depth analysis with examples, counterarguments, and nuanced perspectives on:\n\n{}\n\n{}\n\nPlease include relevant references, examples, and consider multiple viewpoints.",
                    prompt, instructions
                )
            }
            _ => {
                format!("{}\n\n{}", prompt, instructions)
            }
        };
        
        // 更新提示内容
        self.update_user_prompt(json_body, &enhanced_prompt)
    }

    // 翻译提示
    fn translate_prompt(&self, json_body: &mut Value, target_lang: &str) -> bool {
        // 获取原始提示内容
        let original_prompt = self.extract_user_prompt(json_body);
        if original_prompt.is_none() {
            return false;
        }
        
        let prompt = original_prompt.unwrap();
        
        // 注意：实际实现中，这里应该调用翻译API或服务
        // 由于示例的限制，这里只是添加一个简单的指令
        let translated_prompt = format!(
            "Translate the following to {}: \n\n{}",
            target_lang, prompt
        );
        
        // 更新提示内容
        self.update_user_prompt(json_body, &translated_prompt)
    }

    // 分割提示
    fn split_prompt(&self, json_body: &mut Value, max_tokens: usize) -> bool {
        // 获取原始提示内容
        let original_prompt = self.extract_user_prompt(json_body);
        if original_prompt.is_none() {
            return false;
        }
        
        let prompt = original_prompt.unwrap();
        
        // 简单的基于段落的分割
        // 注意：实际实现中，应该使用token计数器来确保准确的分割
        if prompt.len() > max_tokens {
            // 添加分割的提示
            let modified_prompt = format!(
                "This is a long prompt that might exceed token limits. Please respond to the following part first, then ask for the next part if needed:\n\n{}",
                // 简单地取前一部分文本，实际应基于token计数
                &prompt[0..max_tokens.min(prompt.len())]
            );
            
            // 更新提示内容
            return self.update_user_prompt(json_body, &modified_prompt);
        }
        
        false // 无需分割
    }

    // RAG增强
    fn enhance_with_rag(&self, json_body: &mut Value, source: &str, instructions: &str) -> bool {
        // 获取原始提示内容
        let original_prompt = self.extract_user_prompt(json_body);
        if original_prompt.is_none() {
            return false;
        }
        
        let prompt = original_prompt.unwrap();
        
        // 注意：实际实现中，这里应该查询向量数据库或其他知识源
        // 由于示例的限制，这里只是添加一个简单的指令
        let enhanced_prompt = format!(
            "Using the knowledge from {}, please respond to the following:\n\n{}\n\n{}",
            source, prompt, instructions
        );
        
        // 更新提示内容
        self.update_user_prompt(json_body, &enhanced_prompt)
    }

    // 应用自定义转换
    fn apply_custom_transform(&self, json_body: &mut Value, params: &Value, instructions: &str) -> bool {
        // 获取原始提示内容
        let original_prompt = self.extract_user_prompt(json_body);
        if original_prompt.is_none() {
            return false;
        }
        
        let prompt = original_prompt.unwrap();
        
        // 处理自定义参数
        let operation = params.get("operation").and_then(|o| o.as_str()).unwrap_or("append");
        
        let transformed_prompt = match operation {
            "append" => {
                format!("{}\n\n{}", prompt, instructions)
            }
            "prepend" => {
                format!("{}\n\n{}", instructions, prompt)
            }
            "replace" => {
                if let Some(pattern) = params.get("pattern").and_then(|p| p.as_str()) {
                    prompt.replace(pattern, instructions)
                } else {
                    instructions.to_string()
                }
            }
            _ => {
                format!("{}\n\n{}", prompt, instructions)
            }
        };
        
        // 更新提示内容
        self.update_user_prompt(json_body, &transformed_prompt)
    }

    // 提取用户提示内容的辅助方法
    fn extract_user_prompt(&self, json_body: &Value) -> Option<String> {
        // 对OpenAI格式的处理
        if let Some(messages) = json_body.get("messages").and_then(|m| m.as_array()) {
            for message in messages {
                if message.get("role").and_then(|r| r.as_str()) == Some("user") {
                    if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                        return Some(content.to_string());
                    }
                }
            }
        }
        // 对Anthropic格式的处理
        else if let Some(prompt) = json_body.get("prompt").and_then(|p| p.as_str()) {
            if let Some(human_idx) = prompt.find("Human:") {
                return Some(prompt[human_idx + 6..].trim().to_string());
            }
        }
        
        None
    }

    // 更新用户提示内容的辅助方法
    fn update_user_prompt(&self, json_body: &mut Value, new_prompt: &str) -> bool {
        // 对OpenAI格式的处理
        if let Some(messages) = json_body.get_mut("messages").and_then(|m| m.as_array_mut()) {
            for message in messages.iter_mut() {
                if message.get("role").and_then(|r| r.as_str()) == Some("user") {
                    message["content"] = new_prompt.into();
                    return true;
                }
            }
        }
        // 对Anthropic格式的处理
        else if let Some(prompt) = json_body.get_mut("prompt").and_then(|p| p.as_str()) {
            if let Some(human_idx) = prompt.find("Human:") {
                let prefix = &prompt[0..human_idx + 6]; // 包含"Human:"
                let new_content = format!("{} {}", prefix, new_prompt);
                json_body["prompt"] = new_content.into();
                return true;
            }
        }
        
        false
    }
}

// Add new Plugin trait implementation structure
#[async_trait]
impl Plugin for PromptTransformer {
    fn name(&self) -> &'static str {
        "prompt_transformer"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 300, // Prompt transformation should happen after routing
            plugin_type: PluginType::Native,
            description: "Advanced prompt transformation and enhancement for LLM requests".to_string(),
            author: "Proksi Team".to_string(),
            homepage: Some("https://github.com/luizfonseca/proksi".to_string()),
        }
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        if step != PluginStep::ProxyUpstream {
            return Ok((false, None));
        }

        // Get provider name from context (set by LlmRouter)
        let provider = ctx.plugins_data
            .get("llm_provider")
            .and_then(|v| v.as_str());

        // --- TEMPORARY ASSUMPTION: Get request body --- 
        // In a real scenario, this requires specific API support from Pingora
        // or the underlying framework to get mutable access to the body stream.
        // For now, we simulate getting the body. We also need to handle content type.
        let Some(content_type) = session.req_header().headers.get("content-type") else {
            debug!("Skipping prompt transform: No Content-Type header");
            return Ok((false, None));
        };
        if !content_type.to_str().unwrap_or("").contains("application/json") {
            debug!("Skipping prompt transform: Content-Type is not application/json");
            return Ok((false, None));
        }

        // Placeholder for actual body retrieval
        // let mut body_bytes = session.get_request_body_mut()?.to_vec(); // Hypothetical API
        // Using a dummy body for now, as direct access isn't available
        // THIS WILL NOT WORK IN A REAL PROXY SCENARIO WITHOUT CORE CHANGES
        let body_bytes = Bytes::from("{\"messages\":[{\"role\":\"user\",\"content\":\"Hello from dummy body\"}]}");
        warn!("PromptTransformer: Using dummy request body due to limitations in accessing real body.");
        // --- END TEMPORARY ASSUMPTION ---

        let body_str = String::from_utf8_lossy(&body_bytes);
        let mut json_body: Value = match serde_json::from_str(&body_str) {
            Ok(val) => val,
            Err(e) => {
                error!("PromptTransformer: Failed to parse request body as JSON: {}", e);
                return Ok((false, None)); // Don't interrupt flow on parse error
            }
        };

        // Apply transformations
        let mut modified = false;
        for transformation in &self.config.transformations {
            // Check provider applicability
            if let Some(p) = &transformation.provider {
                if let Some(current_provider) = provider {
                    if !current_provider.contains(p) {
                        continue; // Skip transformation if provider doesn't match
                    }
                }
            }

            // Apply the specific transformation
            let applied = match transformation.transform_type {
                 TransformationType::AddSystemMessage => self.add_system_message(&mut json_body, &transformation.content),
                 TransformationType::AddContext => self.add_context(&mut json_body, &transformation.content),
                 TransformationType::AddSafetyCheck => self.add_safety_check(&mut json_body, &transformation.content),
                 TransformationType::ApplyTemplate => transformation.template.as_ref().map_or(false, |t| self.apply_template(&mut json_body, t)),
                 TransformationType::FormatPrompt => transformation.format_style.as_ref().map_or(false, |s| self.format_prompt(&mut json_body, s, &transformation.content)),
                 TransformationType::ExtractKeywords => self.extract_keywords(&mut json_body, &transformation.content),
                 TransformationType::EnhanceContent => self.enhance_content(&mut json_body, transformation.enhancement_level.as_deref().unwrap_or("standard"), &transformation.content),
                 TransformationType::TranslatePrompt => transformation.target_lang.as_ref().map_or(false, |l| self.translate_prompt(&mut json_body, l)),
                 TransformationType::SplitPrompt => self.split_prompt(&mut json_body, transformation.max_tokens.unwrap_or(4000)),
                 TransformationType::RAGEnhancement => transformation.rag_source.as_ref().map_or(false, |s| self.enhance_with_rag(&mut json_body, s, &transformation.content)),
                 TransformationType::Custom => transformation.custom_params.as_ref().map_or(false, |p| self.apply_custom_transform(&mut json_body, p, &transformation.content)),
            };
            modified |= applied;
        }

        // If modified, update the request body and Content-Length header
        if modified {
            debug!("Prompt transformed: {}", json_body);
            match serde_json::to_vec(&json_body) {
                Ok(new_body_bytes_vec) => {
                    let new_body_bytes = Bytes::from(new_body_bytes_vec);
                    let new_len = new_body_bytes.len();

                    // --- TEMPORARY ASSUMPTION: Set request body ---
                    // session.set_request_body(new_body_bytes)?; // Hypothetical API
                    warn!("PromptTransformer: Request body modified but cannot be set due to limitations.");
                    // --- END TEMPORARY ASSUMPTION ---
                    
                    // Update Content-Length header
                    let headers = session.req_header_mut();
                    headers.insert_header("Content-Length", HeaderValue::from_str(&new_len.to_string())?)?;
                    
                    info!("Prompt transformed and Content-Length updated.");
                }
                Err(e) => {
                    error!("PromptTransformer: Failed to serialize modified body: {}", e);
                    // Continue without modifying body if serialization fails
                }
            }
        }

        Ok((false, None)) // Never terminates the request chain itself
    }

    async fn handle_response(
        &self,
        _step: PluginStep,
        _session: &mut Session,
        _ctx: &mut RouterContext,
        _upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        Ok(false) // No response modification needed
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting PromptTransformer plugin");
        info!("PromptTransformer plugin started with {} transformations configured", self.config.transformations.len());
        info!("Available templates: {:?}", self.templates.keys().collect::<Vec<_>>());
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping PromptTransformer plugin");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transformation_type_from_str() {
        assert!(matches!(TransformationType::from("system"), TransformationType::AddSystemMessage));
        assert!(matches!(TransformationType::from("context"), TransformationType::AddContext));
        assert!(matches!(TransformationType::from("safety"), TransformationType::AddSafetyCheck));
        assert!(matches!(TransformationType::from("template"), TransformationType::ApplyTemplate));
        assert!(matches!(TransformationType::from("format"), TransformationType::FormatPrompt));
        assert!(matches!(TransformationType::from("keywords"), TransformationType::ExtractKeywords));
        assert!(matches!(TransformationType::from("enhance"), TransformationType::EnhanceContent));
        assert!(matches!(TransformationType::from("translate"), TransformationType::TranslatePrompt));
        assert!(matches!(TransformationType::from("split"), TransformationType::SplitPrompt));
        assert!(matches!(TransformationType::from("rag"), TransformationType::RAGEnhancement));
        assert!(matches!(TransformationType::from("unknown"), TransformationType::Custom));
    }
    
    #[test]
    fn test_add_system_message() {
        let transformer = PromptTransformer::new();
        
        // OpenAI格式测试
        let mut openai_json = json!({
            "messages": [
                {"role": "user", "content": "Tell me about AI"}
            ]
        });
        
        assert!(transformer.add_system_message(&mut openai_json, "You are a helpful assistant."));
        
        // 解析消息列表并断言
        let messages_count = openai_json["messages"].as_array().unwrap().len();
        assert_eq!(messages_count, 2);
        
        let role = openai_json["messages"][0]["role"].as_str().unwrap();
        let content = openai_json["messages"][0]["content"].as_str().unwrap();
        assert_eq!(role, "system");
        assert_eq!(content, "You are a helpful assistant.");
        
        // 已有系统消息，不应再添加
        assert!(!transformer.add_system_message(&mut openai_json, "Another system message."));
        
        // 验证消息数量仍然是2
        let final_count = openai_json["messages"].as_array().unwrap().len();
        assert_eq!(final_count, 2);
    }
    
    #[test]
    fn test_format_prompt() {
        let transformer = PromptTransformer::new();
        
        // OpenAI格式测试
        let mut openai_json = json!({
            "messages": [
                {"role": "user", "content": "Tell me about AI. Include its history. Discuss potential risks."}
            ]
        });
        
        // 测试项目符号格式
        assert!(transformer.format_prompt(&mut openai_json, "bullet", "Bullet points"));
        let content = openai_json["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("• Tell me about AI"));
        assert!(content.contains("• Include its history"));
        assert!(content.contains("• Discuss potential risks"));
        
        // 测试编号格式
        let mut openai_json = json!({
            "messages": [
                {"role": "user", "content": "Tell me about AI. Include its history. Discuss potential risks."}
            ]
        });
        
        assert!(transformer.format_prompt(&mut openai_json, "numbered", "Numbered list"));
        let content = openai_json["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("1. Tell me about AI"));
        assert!(content.contains("2. Include its history"));
        assert!(content.contains("3. Discuss potential risks"));
    }
    
    #[test]
    fn test_extract_keywords() {
        let transformer = PromptTransformer::new();
        
        // OpenAI格式测试
        let mut openai_json = json!({
            "messages": [
                {"role": "user", "content": "Tell me about artificial intelligence and machine learning applications in healthcare"}
            ]
        });
        
        assert!(transformer.extract_keywords(&mut openai_json, "Focus on these keywords"));
        let content = openai_json["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("Keywords:"));
        assert!(content.contains("artificial"));
        assert!(content.contains("intelligence"));
        assert!(content.contains("machine"));
        assert!(content.contains("learning"));
        assert!(content.contains("healthcare"));
        assert!(content.contains("Original request:"));
    }
    
    #[test]
    fn test_enhance_content() {
        let transformer = PromptTransformer::new();
        
        // OpenAI格式测试 - 基础增强
        let mut openai_json = json!({
            "messages": [
                {"role": "user", "content": "Tell me about AI"}
            ]
        });
        
        assert!(transformer.enhance_content(&mut openai_json, "basic", "Include examples"));
        let content = openai_json["messages"][0]["content"].as_str().unwrap();
        assert_eq!(content, "Tell me about AI\n\nInclude examples");
        
        // OpenAI格式测试 - 高级增强
        let mut openai_json = json!({
            "messages": [
                {"role": "user", "content": "Tell me about AI"}
            ]
        });
        
        assert!(transformer.enhance_content(&mut openai_json, "advanced", "Include examples"));
        let content = openai_json["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("expert in this subject"));
        assert!(content.contains("in-depth analysis"));
        assert!(content.contains("Include examples"));
    }
    
    #[test]
    fn test_translate_prompt() {
        let transformer = PromptTransformer::new();
        
        // OpenAI格式测试
        let mut openai_json = json!({
            "messages": [
                {"role": "user", "content": "Tell me about AI"}
            ]
        });
        
        assert!(transformer.translate_prompt(&mut openai_json, "Chinese"));
        let content = openai_json["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("Translate the following to Chinese:"));
        assert!(content.contains("Tell me about AI"));
    }
    
    #[test]
    fn test_extract_and_update_user_prompt() {
        let transformer = PromptTransformer::new();
        
        // OpenAI格式测试
        let openai_json = json!({
            "messages": [
                {"role": "user", "content": "Original prompt"}
            ]
        });
        
        let prompt = transformer.extract_user_prompt(&openai_json);
        assert_eq!(prompt, Some("Original prompt".to_string()));
        
        let mut openai_json_mut = openai_json.clone();
        assert!(transformer.update_user_prompt(&mut openai_json_mut, "Updated prompt"));
        let updated_content = openai_json_mut["messages"][0]["content"].as_str().unwrap();
        assert_eq!(updated_content, "Updated prompt");
        
        // Anthropic格式测试
        let anthropic_json = json!({
            "prompt": "Human: Original anthropic prompt"
        });
        
        let prompt = transformer.extract_user_prompt(&anthropic_json);
        assert_eq!(prompt, Some("Original anthropic prompt".to_string()));
        
        let mut anthropic_json_mut = anthropic_json.clone();
        assert!(transformer.update_user_prompt(&mut anthropic_json_mut, "Updated anthropic prompt"));
        let updated_content = anthropic_json_mut["prompt"].as_str().unwrap();
        assert!(updated_content.contains("Human: Updated anthropic prompt"));
    }
    
    // Add a test to verify creating the plugin with config
    #[test]
    fn test_plugin_creation_with_config() {
        let config = PromptTransformConfig {
            transformations: vec![
                PromptTransformation {
                    transform_type: TransformationType::AddSystemMessage,
                    provider: Some("openai".to_string()),
                    content: "Test System Message".to_string(),
                    template: None,
                    target_lang: None,
                    format_style: None,
                    max_tokens: None,
                    enhancement_level: None,
                    rag_source: None,
                    custom_params: None,
                }
            ]
        };
        
        let plugin = PromptTransformer::with_config(config.clone(), None);
        
        assert_eq!(plugin.config.transformations.len(), 1);
        assert!(matches!(plugin.config.transformations[0].transform_type, TransformationType::AddSystemMessage));
        assert!(plugin.templates.contains_key("general_instruction")); // Check default templates loaded
    }
    
    // NOTE: Testing the full handle_request flow is currently limited
    // due to the inability to mock/access the request body easily in tests
    // within the current structure. The existing tests cover the core
    // transformation logic helpers.
} 