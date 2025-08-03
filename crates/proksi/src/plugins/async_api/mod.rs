use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use pingora::{http::ResponseHeader, proxy::Session};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    plugins::core::{Plugin, PluginError, PluginStep},
    proxy_server::{https_proxy::RouterContext, HttpResponse},
};

// 异步API支持插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncApiConfig {
    pub enabled: bool,
    pub message_brokers: Vec<MessageBrokerConfig>,
    pub event_handlers: Vec<EventHandlerConfig>,
    pub webhook_config: WebhookConfig,
    pub retry_config: RetryConfig,
    pub dead_letter_config: DeadLetterConfig,
}

// 消息代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBrokerConfig {
    pub name: String,
    pub broker_type: BrokerType,
    pub connection_string: String,
    pub topics: Vec<TopicConfig>,
    pub consumer_groups: Vec<ConsumerGroupConfig>,
    pub producer_config: ProducerConfig,
    pub enabled: bool,
}

// 代理类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrokerType {
    Kafka,
    RabbitMQ,
    Redis,
    NATS,
    Custom(String),
}

// 主题配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    pub name: String,
    pub partitions: Option<u32>,
    pub replication_factor: Option<u16>,
    pub retention_ms: Option<u64>,
    pub cleanup_policy: Option<String>,
}

// 消费者组配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerGroupConfig {
    pub group_id: String,
    pub topics: Vec<String>,
    pub auto_offset_reset: String,
    pub enable_auto_commit: bool,
    pub max_poll_records: u32,
}

// 生产者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerConfig {
    pub acks: String,
    pub retries: u32,
    pub batch_size: u32,
    pub linger_ms: u64,
    pub buffer_memory: u64,
}

// 事件处理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandlerConfig {
    pub name: String,
    pub event_type: String,
    pub handler_type: EventHandlerType,
    pub endpoint: Option<String>,
    pub headers: HashMap<String, String>,
    pub timeout_ms: u64,
    pub enabled: bool,
}

// 事件处理器类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventHandlerType {
    Webhook,
    MessageQueue,
    Database,
    Custom(String),
}

// Webhook配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub default_timeout_ms: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub signature_header: Option<String>,
    pub secret_key: Option<String>,
}

// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub enabled: bool,
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub retry_on_status_codes: Vec<u16>,
}

// 死信队列配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterConfig {
    pub enabled: bool,
    pub topic_suffix: String,
    pub max_retries: u32,
    pub retention_hours: u64,
}

// 异步消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncMessage {
    pub id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub correlation_id: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
}

// 消息状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    DeadLetter,
}

// 消息处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResult {
    pub message_id: String,
    pub status: MessageStatus,
    pub processed_at: DateTime<Utc>,
    pub error: Option<String>,
    pub retry_count: u32,
    pub processing_time_ms: u64,
}

impl Default for AsyncApiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            message_brokers: vec![],
            event_handlers: vec![],
            webhook_config: WebhookConfig::default(),
            retry_config: RetryConfig::default(),
            dead_letter_config: DeadLetterConfig::default(),
        }
    }
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_timeout_ms: 30000,
            max_retries: 3,
            retry_delay_ms: 1000,
            signature_header: Some("X-Signature".to_string()),
            secret_key: None,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
            backoff_multiplier: 2.0,
            retry_on_status_codes: vec![500, 502, 503, 504],
        }
    }
}

impl Default for DeadLetterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            topic_suffix: "_dlq".to_string(),
            max_retries: 5,
            retention_hours: 168, // 7 days
        }
    }
}

// 异步API插件主结构
#[derive(Debug)]
pub struct AsyncApiPlugin {
    config: Arc<RwLock<AsyncApiConfig>>,
    message_queue: Arc<Mutex<Vec<AsyncMessage>>>,
    processing_queue: Arc<Mutex<Vec<AsyncMessage>>>,
    dead_letter_queue: Arc<Mutex<Vec<AsyncMessage>>>,
    message_results: Arc<Mutex<HashMap<String, MessageResult>>>,
    // 简化的消息代理连接（实际实现中应该是真实的连接）
    broker_connections: Arc<Mutex<HashMap<String, bool>>>,
}

impl Default for AsyncApiPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncApiPlugin {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AsyncApiConfig::default())),
            message_queue: Arc::new(Mutex::new(Vec::new())),
            processing_queue: Arc::new(Mutex::new(Vec::new())),
            dead_letter_queue: Arc::new(Mutex::new(Vec::new())),
            message_results: Arc::new(Mutex::new(HashMap::new())),
            broker_connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_config(config: AsyncApiConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            message_queue: Arc::new(Mutex::new(Vec::new())),
            processing_queue: Arc::new(Mutex::new(Vec::new())),
            dead_letter_queue: Arc::new(Mutex::new(Vec::new())),
            message_results: Arc::new(Mutex::new(HashMap::new())),
            broker_connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // 发布异步消息
    pub async fn publish_message(&self, message: AsyncMessage) -> Result<String> {
        let config = self.config.read().await;

        if !config.enabled {
            return Err(anyhow::anyhow!("Async API plugin is disabled"));
        }

        let message_id = message.id.clone();
        info!("Publishing async message: {}", message_id);

        // 添加到消息队列
        let mut queue = self.message_queue.lock().await;
        queue.push(message.clone());

        // 记录消息状态
        let mut results = self.message_results.lock().await;
        results.insert(message_id.clone(), MessageResult {
            message_id: message_id.clone(),
            status: MessageStatus::Pending,
            processed_at: Utc::now(),
            error: None,
            retry_count: 0,
            processing_time_ms: 0,
        });

        // 触发异步处理
        self.process_message_async(message).await?;

        Ok(message_id)
    }

    // 异步处理消息
    async fn process_message_async(&self, message: AsyncMessage) -> Result<()> {
        let config = self.config.read().await;
        let start_time = std::time::Instant::now();

        // 移动到处理队列
        {
            let mut processing = self.processing_queue.lock().await;
            processing.push(message.clone());
        }

        // 更新状态为处理中
        {
            let mut results = self.message_results.lock().await;
            if let Some(result) = results.get_mut(&message.id) {
                result.status = MessageStatus::Processing;
                result.processed_at = Utc::now();
            }
        }

        // 查找匹配的事件处理器
        let matching_handlers: Vec<_> = config.event_handlers.iter()
            .filter(|handler| handler.enabled && handler.event_type == message.event_type)
            .collect();

        if matching_handlers.is_empty() {
            warn!("No handlers found for event type: {}", message.event_type);
            self.mark_message_failed(&message.id, "No matching handlers found").await;
            return Ok(());
        }

        // 处理每个匹配的处理器
        let mut all_successful = true;
        let mut last_error = None;

        for handler in matching_handlers {
            match self.execute_handler(handler, &message).await {
                Ok(_) => {
                    info!("Handler {} processed message {} successfully", handler.name, message.id);
                }
                Err(e) => {
                    error!("Handler {} failed to process message {}: {}", handler.name, message.id, e);
                    all_successful = false;
                    last_error = Some(e.to_string());
                }
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        // 更新最终状态
        if all_successful {
            self.mark_message_completed(&message.id, processing_time).await;
        } else {
            self.handle_message_failure(&message, last_error.unwrap_or_default()).await?;
        }

        Ok(())
    }

    // 执行事件处理器
    async fn execute_handler(&self, handler: &EventHandlerConfig, message: &AsyncMessage) -> Result<()> {
        match handler.handler_type {
            EventHandlerType::Webhook => {
                self.execute_webhook_handler(handler, message).await
            }
            EventHandlerType::MessageQueue => {
                self.execute_message_queue_handler(handler, message).await
            }
            EventHandlerType::Database => {
                self.execute_database_handler(handler, message).await
            }
            EventHandlerType::Custom(ref custom_type) => {
                self.execute_custom_handler(custom_type, handler, message).await
            }
        }
    }

    // 执行Webhook处理器
    async fn execute_webhook_handler(&self, handler: &EventHandlerConfig, message: &AsyncMessage) -> Result<()> {
        let endpoint = handler.endpoint.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Webhook endpoint not configured"))?;

        let client = reqwest::Client::new();
        let timeout = Duration::from_millis(handler.timeout_ms);

        let mut request_builder = client
            .post(endpoint)
            .timeout(timeout)
            .json(&message.payload);

        // 添加自定义头部
        for (key, value) in &handler.headers {
            request_builder = request_builder.header(key, value);
        }

        // 添加标准头部
        request_builder = request_builder
            .header("X-Event-Type", &message.event_type)
            .header("X-Message-Id", &message.id)
            .header("X-Timestamp", message.timestamp.to_rfc3339());

        if let Some(correlation_id) = &message.correlation_id {
            request_builder = request_builder.header("X-Correlation-Id", correlation_id);
        }

        let response = request_builder.send().await?;

        if response.status().is_success() {
            info!("Webhook {} executed successfully for message {}", endpoint, message.id);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Webhook failed with status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ))
        }
    }

    // 执行消息队列处理器
    async fn execute_message_queue_handler(&self, handler: &EventHandlerConfig, message: &AsyncMessage) -> Result<()> {
        // 这里应该实现实际的消息队列发送逻辑
        // 目前是一个占位符实现
        info!("Executing message queue handler {} for message {}", handler.name, message.id);

        // 模拟消息队列发送
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(())
    }

    // 执行数据库处理器
    async fn execute_database_handler(&self, handler: &EventHandlerConfig, message: &AsyncMessage) -> Result<()> {
        // 这里应该实现实际的数据库写入逻辑
        // 目前是一个占位符实现
        info!("Executing database handler {} for message {}", handler.name, message.id);

        // 模拟数据库写入
        tokio::time::sleep(Duration::from_millis(5)).await;

        Ok(())
    }

    // 执行自定义处理器
    async fn execute_custom_handler(&self, custom_type: &str, handler: &EventHandlerConfig, message: &AsyncMessage) -> Result<()> {
        info!("Executing custom handler {} (type: {}) for message {}", handler.name, custom_type, message.id);

        // 这里可以根据custom_type实现不同的自定义逻辑
        match custom_type {
            "log" => {
                info!("Custom log handler: {:?}", message);
                Ok(())
            }
            "metrics" => {
                info!("Custom metrics handler: collecting metrics for message {}", message.id);
                Ok(())
            }
            _ => {
                warn!("Unknown custom handler type: {}", custom_type);
                Ok(())
            }
        }
    }

    // 标记消息完成
    async fn mark_message_completed(&self, message_id: &str, processing_time_ms: u64) {
        let mut results = self.message_results.lock().await;
        if let Some(result) = results.get_mut(message_id) {
            result.status = MessageStatus::Completed;
            result.processed_at = Utc::now();
            result.processing_time_ms = processing_time_ms;
        }

        // 从处理队列中移除
        let mut processing = self.processing_queue.lock().await;
        processing.retain(|msg| msg.id != message_id);

        info!("Message {} completed successfully in {}ms", message_id, processing_time_ms);
    }

    // 标记消息失败
    async fn mark_message_failed(&self, message_id: &str, error: &str) {
        let mut results = self.message_results.lock().await;
        if let Some(result) = results.get_mut(message_id) {
            result.status = MessageStatus::Failed;
            result.processed_at = Utc::now();
            result.error = Some(error.to_string());
        }

        // 从处理队列中移除
        let mut processing = self.processing_queue.lock().await;
        processing.retain(|msg| msg.id != message_id);

        error!("Message {} failed: {}", message_id, error);
    }

    // 处理消息失败（包含重试逻辑）
    async fn handle_message_failure(&self, message: &AsyncMessage, error: String) -> Result<()> {
        let config = self.config.read().await;

        if !config.retry_config.enabled {
            self.mark_message_failed(&message.id, &error).await;
            return Ok(());
        }

        let mut updated_message = message.clone();
        updated_message.retry_count += 1;

        if updated_message.retry_count >= config.retry_config.max_attempts {
            // 超过最大重试次数，移到死信队列
            if config.dead_letter_config.enabled {
                self.move_to_dead_letter_queue(updated_message).await;
            } else {
                self.mark_message_failed(&message.id, &format!("Max retries exceeded: {}", error)).await;
            }
            return Ok(());
        }

        // 计算重试延迟
        let delay_ms = self.calculate_retry_delay(&config.retry_config, updated_message.retry_count);

        info!("Retrying message {} (attempt {}/{}) after {}ms delay",
              message.id, updated_message.retry_count, config.retry_config.max_attempts, delay_ms);

        // 更新重试计数
        {
            let mut results = self.message_results.lock().await;
            if let Some(result) = results.get_mut(&message.id) {
                result.retry_count = updated_message.retry_count;
                result.error = Some(format!("Retry {}: {}", updated_message.retry_count, error));
            }
        }

        // 简化重试逻辑：直接将消息重新加入队列
        info!("Scheduling retry for message {} after {}ms", message.id, delay_ms);

        // 将消息重新加入队列（实际实现中可以使用更复杂的调度机制）
        let mut queue = self.message_queue.lock().await;
        queue.push(updated_message);

        Ok(())
    }

    // 计算重试延迟
    fn calculate_retry_delay(&self, retry_config: &RetryConfig, attempt: u32) -> u64 {
        let base_delay = retry_config.initial_delay_ms as f64;
        let multiplier = retry_config.backoff_multiplier;
        let max_delay = retry_config.max_delay_ms;

        let calculated_delay = base_delay * multiplier.powi(attempt as i32 - 1);
        calculated_delay.min(max_delay as f64) as u64
    }

    // 移动到死信队列
    async fn move_to_dead_letter_queue(&self, message: AsyncMessage) {
        let mut dlq = self.dead_letter_queue.lock().await;
        dlq.push(message.clone());

        // 更新状态
        let mut results = self.message_results.lock().await;
        if let Some(result) = results.get_mut(&message.id) {
            result.status = MessageStatus::DeadLetter;
            result.processed_at = Utc::now();
        }

        warn!("Message {} moved to dead letter queue after {} retries", message.id, message.retry_count);
    }

    // 获取消息状态
    pub async fn get_message_status(&self, message_id: &str) -> Option<MessageResult> {
        let results = self.message_results.lock().await;
        results.get(message_id).cloned()
    }

    // 获取队列统计信息
    pub async fn get_queue_stats(&self) -> serde_json::Value {
        let pending_count = self.message_queue.lock().await.len();
        let processing_count = self.processing_queue.lock().await.len();
        let dlq_count = self.dead_letter_queue.lock().await.len();

        let results = self.message_results.lock().await;
        let completed_count = results.values().filter(|r| matches!(r.status, MessageStatus::Completed)).count();
        let failed_count = results.values().filter(|r| matches!(r.status, MessageStatus::Failed)).count();

        serde_json::json!({
            "pending_messages": pending_count,
            "processing_messages": processing_count,
            "completed_messages": completed_count,
            "failed_messages": failed_count,
            "dead_letter_messages": dlq_count,
            "total_messages": results.len()
        })
    }

    // 清理过期消息
    pub async fn cleanup_expired_messages(&self) -> Result<()> {
        let config = self.config.read().await;
        let retention_duration = chrono::Duration::hours(config.dead_letter_config.retention_hours as i64);
        let cutoff_time = Utc::now() - retention_duration;

        // 清理死信队列中的过期消息
        {
            let mut dlq = self.dead_letter_queue.lock().await;
            let initial_count = dlq.len();
            dlq.retain(|msg| msg.timestamp > cutoff_time);
            let removed_count = initial_count - dlq.len();
            if removed_count > 0 {
                info!("Cleaned up {} expired messages from dead letter queue", removed_count);
            }
        }

        // 清理过期的消息结果
        {
            let mut results = self.message_results.lock().await;
            let initial_count = results.len();
            results.retain(|_, result| result.processed_at > cutoff_time);
            let removed_count = initial_count - results.len();
            if removed_count > 0 {
                info!("Cleaned up {} expired message results", removed_count);
            }
        }

        Ok(())
    }

    // 创建异步消息的辅助方法
    pub fn create_message(
        event_type: String,
        payload: serde_json::Value,
        source: String,
        correlation_id: Option<String>,
    ) -> AsyncMessage {
        AsyncMessage {
            id: Uuid::new_v4().to_string(),
            event_type,
            payload,
            headers: HashMap::new(),
            timestamp: Utc::now(),
            source,
            correlation_id,
            retry_count: 0,
            max_retries: 3,
        }
    }
}

// 实现Clone trait以支持异步任务
impl Clone for AsyncApiPlugin {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            message_queue: Arc::clone(&self.message_queue),
            processing_queue: Arc::clone(&self.processing_queue),
            dead_letter_queue: Arc::clone(&self.dead_letter_queue),
            message_results: Arc::clone(&self.message_results),
            broker_connections: Arc::clone(&self.broker_connections),
        }
    }
}

#[async_trait]
impl Plugin for AsyncApiPlugin {
    fn name(&self) -> &'static str {
        "async_api"
    }

    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // 异步API插件主要在请求阶段检查是否需要异步处理
        if step != PluginStep::Request {
            return Ok((false, None));
        }

        let config = self.config.read().await;
        if !config.enabled {
            return Ok((false, None));
        }

        // 检查是否是异步API请求
        let is_async_request = session.req_header().headers
            .get("x-async-processing")
            .map(|v| v.to_str().unwrap_or("false") == "true")
            .unwrap_or(false);

        if !is_async_request {
            return Ok((false, None));
        }

        // 处理异步请求
        let request_id = ctx.request_id.clone();
        let event_type = session.req_header().headers
            .get("x-event-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("generic")
            .to_string();

        // 创建异步消息（这里简化处理，实际应该从请求体中提取payload）
        let message = Self::create_message(
            event_type,
            serde_json::json!({
                "request_id": request_id,
                "path": session.req_header().uri.path(),
                "method": session.req_header().method.as_str(),
                "timestamp": Utc::now().to_rfc3339()
            }),
            "proksi_gateway".to_string(),
            Some(request_id.clone()),
        );

        // 发布异步消息
        match self.publish_message(message).await {
            Ok(message_id) => {
                info!("Async message published: {}", message_id);

                // 返回异步响应
                let response_body = serde_json::json!({
                    "status": "accepted",
                    "message_id": message_id,
                    "request_id": request_id
                });

                let mut headers = HeaderMap::new();
                headers.insert(HeaderName::from_static("content-type"), HeaderValue::from_static("application/json"));
                headers.insert(HeaderName::from_static("x-message-id"),
                              HeaderValue::from_str(&message_id).unwrap_or_else(|_| HeaderValue::from_static("unknown")));

                let response = HttpResponse {
                    status_code: StatusCode::ACCEPTED,
                    headers,
                    body: Bytes::from(serde_json::to_vec(&response_body)?),
                };

                Ok((true, Some(response)))
            }
            Err(e) => {
                error!("Failed to publish async message: {}", e);

                let error_response = serde_json::json!({
                    "error": "Failed to process async request",
                    "details": e.to_string()
                });

                let mut headers = HeaderMap::new();
                headers.insert(HeaderName::from_static("content-type"), HeaderValue::from_static("application/json"));

                let response = HttpResponse {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    headers,
                    body: Bytes::from(serde_json::to_vec(&error_response)?),
                };

                Ok((true, Some(response)))
            }
        }
    }

    async fn handle_response(
        &self,
        _step: PluginStep,
        _session: &mut Session,
        _ctx: &mut RouterContext,
        _upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        // 异步API插件在响应阶段不需要特殊处理
        Ok(false)
    }

    async fn start(&mut self) -> Result<(), PluginError> {
        info!("Starting Async API plugin");

        let config = self.config.read().await;
        if !config.enabled {
            info!("Async API plugin is disabled");
            return Ok(());
        }

        // 初始化消息代理连接（占位符实现）
        for broker in &config.message_brokers {
            if broker.enabled {
                info!("Initializing connection to {} broker: {}",
                      broker.broker_type.to_string(), broker.name);

                // 这里应该实现实际的连接逻辑
                let mut connections = self.broker_connections.lock().await;
                connections.insert(broker.name.clone(), true);
            }
        }

        // 启动清理任务
        let plugin_clone = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600)); // 每小时清理一次
            loop {
                interval.tick().await;
                if let Err(e) = plugin_clone.cleanup_expired_messages().await {
                    error!("Failed to cleanup expired messages: {}", e);
                }
            }
        });

        info!("Async API plugin started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PluginError> {
        info!("Stopping Async API plugin");

        // 关闭消息代理连接
        let mut connections = self.broker_connections.lock().await;
        connections.clear();

        info!("Async API plugin stopped");
        Ok(())
    }
}

impl std::fmt::Display for BrokerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrokerType::Kafka => write!(f, "Kafka"),
            BrokerType::RabbitMQ => write!(f, "RabbitMQ"),
            BrokerType::Redis => write!(f, "Redis"),
            BrokerType::NATS => write!(f, "NATS"),
            BrokerType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_async_message_creation() {
        let message = AsyncApiPlugin::create_message(
            "test_event".to_string(),
            serde_json::json!({"key": "value"}),
            "test_source".to_string(),
            Some("correlation_123".to_string()),
        );

        assert_eq!(message.event_type, "test_event");
        assert_eq!(message.source, "test_source");
        assert_eq!(message.correlation_id, Some("correlation_123".to_string()));
        assert_eq!(message.retry_count, 0);
        assert!(!message.id.is_empty());
    }

    #[tokio::test]
    async fn test_message_publishing() {
        let mut config = AsyncApiConfig::default();
        config.enabled = false; // 禁用插件避免实际处理

        let plugin = AsyncApiPlugin::with_config(config);

        let message = AsyncApiPlugin::create_message(
            "test_event".to_string(),
            serde_json::json!({"data": "test"}),
            "test".to_string(),
            None,
        );

        let _message_id = message.id.clone();
        let result = plugin.publish_message(message).await;

        // 当插件禁用时，应该返回错误
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let mut config = AsyncApiConfig::default();
        config.event_handlers = vec![]; // 清空事件处理器避免实际处理

        let plugin = AsyncApiPlugin::with_config(config);

        // 直接添加消息到结果中而不是发布（避免异步处理）
        {
            let mut results = plugin.message_results.lock().await;
            for i in 0..3 {
                let message_id = format!("test_message_{}", i);
                results.insert(message_id.clone(), MessageResult {
                    message_id,
                    status: MessageStatus::Completed,
                    processed_at: Utc::now(),
                    error: None,
                    retry_count: 0,
                    processing_time_ms: 100,
                });
            }
        }

        let stats = plugin.get_queue_stats().await;
        assert_eq!(stats["total_messages"].as_u64().unwrap(), 3);
        assert_eq!(stats["completed_messages"].as_u64().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_retry_delay_calculation() {
        let plugin = AsyncApiPlugin::new();
        let retry_config = RetryConfig {
            enabled: true,
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            retry_on_status_codes: vec![500, 502, 503],
        };

        // 测试指数退避
        let delay1 = plugin.calculate_retry_delay(&retry_config, 1);
        let delay2 = plugin.calculate_retry_delay(&retry_config, 2);
        let delay3 = plugin.calculate_retry_delay(&retry_config, 3);

        assert_eq!(delay1, 1000);  // 1000 * 2^0
        assert_eq!(delay2, 2000);  // 1000 * 2^1
        assert_eq!(delay3, 4000);  // 1000 * 2^2

        // 测试最大延迟限制
        let delay_max = plugin.calculate_retry_delay(&retry_config, 10);
        assert_eq!(delay_max, 30000);
    }

    #[tokio::test]
    async fn test_webhook_handler_config() {
        let handler = EventHandlerConfig {
            name: "test_webhook".to_string(),
            event_type: "test_event".to_string(),
            handler_type: EventHandlerType::Webhook,
            endpoint: Some("https://example.com/webhook".to_string()),
            headers: [("Authorization".to_string(), "Bearer token".to_string())]
                .into_iter().collect(),
            timeout_ms: 5000,
            enabled: true,
        };

        assert_eq!(handler.name, "test_webhook");
        assert!(matches!(handler.handler_type, EventHandlerType::Webhook));
        assert!(handler.endpoint.is_some());
        assert!(handler.headers.contains_key("Authorization"));
    }

    #[tokio::test]
    async fn test_message_broker_config() {
        let broker = MessageBrokerConfig {
            name: "test_kafka".to_string(),
            broker_type: BrokerType::Kafka,
            connection_string: "localhost:9092".to_string(),
            topics: vec![TopicConfig {
                name: "test_topic".to_string(),
                partitions: Some(3),
                replication_factor: Some(1),
                retention_ms: Some(86400000), // 1 day
                cleanup_policy: Some("delete".to_string()),
            }],
            consumer_groups: vec![ConsumerGroupConfig {
                group_id: "test_group".to_string(),
                topics: vec!["test_topic".to_string()],
                auto_offset_reset: "earliest".to_string(),
                enable_auto_commit: true,
                max_poll_records: 500,
            }],
            producer_config: ProducerConfig {
                acks: "all".to_string(),
                retries: 3,
                batch_size: 16384,
                linger_ms: 5,
                buffer_memory: 33554432,
            },
            enabled: true,
        };

        assert_eq!(broker.name, "test_kafka");
        assert!(matches!(broker.broker_type, BrokerType::Kafka));
        assert_eq!(broker.topics.len(), 1);
        assert_eq!(broker.consumer_groups.len(), 1);
        assert!(broker.enabled);
    }

    #[tokio::test]
    async fn test_cleanup_expired_messages() {
        let mut config = AsyncApiConfig::default();
        config.dead_letter_config.retention_hours = 0; // 立即过期

        let plugin = AsyncApiPlugin::with_config(config);

        // 添加一些消息到死信队列
        {
            let mut dlq = plugin.dead_letter_queue.lock().await;
            let old_message = AsyncMessage {
                id: "old_message".to_string(),
                event_type: "test".to_string(),
                payload: serde_json::json!({}),
                headers: HashMap::new(),
                timestamp: Utc::now() - chrono::Duration::hours(2), // 2小时前
                source: "test".to_string(),
                correlation_id: None,
                retry_count: 0,
                max_retries: 3,
            };
            dlq.push(old_message);
        }

        // 等待一小段时间确保消息过期
        sleep(Duration::from_millis(10)).await;

        // 执行清理
        let result = plugin.cleanup_expired_messages().await;
        assert!(result.is_ok());

        // 检查死信队列是否为空
        let dlq = plugin.dead_letter_queue.lock().await;
        assert_eq!(dlq.len(), 0);
    }
}
