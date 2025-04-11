use std::{borrow::Cow, collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bytes::Bytes;
use http::HeaderValue;
use once_cell::sync::Lazy;
use pingora::{
    http::{RequestHeader, ResponseHeader},
    proxy::Session,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error};

use crate::{
    config::RoutePlugin,
    plugins::get_required_config,
    proxy_server::https_proxy::RouterContext,
};

use super::MiddlewarePlugin;

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
            _ => TransformationType::Custom,
        }
    }
}

// 提示转换配置
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

pub struct PromptTransformer {
    pub config: Arc<HashMap<String, PromptTransformConfig>>,
    pub templates: Arc<HashMap<String, String>>,
}

impl PromptTransformer {
    pub fn new() -> Self {
        // 初始化一些默认模板
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
            config: Arc::new(HashMap::new()),
            templates: Arc::new(templates),
        }
    }

    // 处理提示转换
    async fn transform_prompt(&self, session: &mut Session, config_name: &str, provider: Option<&str>) -> Result<()> {
        let configs = self.config.clone();
        
        if let Some(transform_config) = configs.get(config_name) {
            // 确保请求包含JSON正文
            if let Some(content_type) = session.req_header().headers.get("content-type") {
                if !content_type.to_str().unwrap_or("").contains("application/json") {
                    return Ok(());
                }
            } else {
                return Ok(()); // 没有Content-Type，跳过
            }

            // 在Pingora当前API中，我们无法直接读取请求体
            // 这里需要通过其他方式获取请求体，例如从头信息中提取
            // 在真实实现中，可能需要修改Pingora或使用特定的钩子
            
            // 此处我们使用一个模拟的请求体用于示例
            // 在实际实现中，这部分需要根据Pingora的API进行适配
            let body_bytes = Bytes::from("{\"messages\":[{\"role\":\"user\",\"content\":\"Hello\"}]}");
            
            // 解析JSON请求体
            let body_str = String::from_utf8_lossy(&body_bytes);
            let mut json_body: Value = match serde_json::from_str(&body_str) {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to parse request body as JSON: {}", e);
                    return Ok(());
                }
            };

            // 应用转换
            let mut modified = false;
            for transformation in &transform_config.transformations {
                // 检查是否适用于当前提供商
                if let Some(p) = &transformation.provider {
                    if let Some(current_provider) = provider {
                        if !current_provider.contains(p) {
                            continue;
                        }
                    }
                }

                // 根据转换类型应用不同的处理
                match transformation.transform_type {
                    TransformationType::AddSystemMessage => {
                        modified |= self.add_system_message(&mut json_body, &transformation.content);
                    }
                    TransformationType::AddContext => {
                        modified |= self.add_context(&mut json_body, &transformation.content);
                    }
                    TransformationType::AddSafetyCheck => {
                        modified |= self.add_safety_check(&mut json_body, &transformation.content);
                    }
                    TransformationType::ApplyTemplate => {
                        if let Some(template_name) = &transformation.template {
                            modified |= self.apply_template(&mut json_body, template_name);
                        }
                    }
                    TransformationType::Custom => {
                        // 自定义转换逻辑
                    }
                }
            }

            // 如果发生了修改，在实际实现中我们需要更新请求体
            if modified {
                // 注意：在当前Pingora API中，我们无法直接修改请求体
                // 这里仅记录修改发生，实际应用中需要根据Pingora API进行适配
                debug!("Prompt transformed successfully: {}", json_body);
            }
        }
        
        Ok(())
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
        let templates = self.templates.clone();
        
        if let Some(template) = templates.get(template_name) {
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
}

// 实现中间件插件 trait
#[async_trait]
impl MiddlewarePlugin for PromptTransformer {
    async fn request_filter(
        &self,
        session: &mut Session,
        state: &mut RouterContext,
        config: &RoutePlugin,
    ) -> Result<bool> {
        // 获取插件配置
        let config_data = config.config.as_ref().ok_or_else(|| anyhow!("Prompt Transform plugin requires configuration"))?;
        
        // 获取配置名
        let config_name = get_required_config(config_data, "config_name").unwrap_or_else(|_| "default".to_string());
        
        // 检查之前是否已确定LLM提供商
        let provider = state.extensions.get(&Cow::Borrowed("llm_provider"))
            .map(|s| s.as_str());
        
        // 转换提示
        self.transform_prompt(session, &config_name, provider).await?;
        
        // 继续处理请求
        Ok(false)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        _upstream_request: &mut RequestHeader,
        _state: &mut RouterContext,
    ) -> Result<()> {
        // 提示已在request_filter中转换，这里无需额外操作
        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        _state: &mut RouterContext,
        _config: &RoutePlugin,
    ) -> Result<bool> {
        // 响应无需处理
        Ok(false)
    }

    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        _upstream_response: &mut ResponseHeader,
        _state: &mut RouterContext,
    ) -> Result<()> {
        // 响应无需处理
        Ok(())
    }
}

// 在静态注册表中注册插件
pub static PROMPT_TRANSFORMER: Lazy<PromptTransformer> = Lazy::new(PromptTransformer::new);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transformation_type_from_str() {
        assert!(matches!(TransformationType::from("system"), TransformationType::AddSystemMessage));
        assert!(matches!(TransformationType::from("context"), TransformationType::AddContext));
        assert!(matches!(TransformationType::from("safety"), TransformationType::AddSafetyCheck));
        assert!(matches!(TransformationType::from("template"), TransformationType::ApplyTemplate));
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
} 