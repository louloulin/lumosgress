# Proksi异步API使用指南

Proksi的异步API支持允许您处理长时间运行的AI请求，而无需保持连接等待响应。这对于批量处理、后台任务和高吞吐量场景特别有用。

## 概述

异步API支持以下特性：

- **消息队列集成**：支持Kafka、RabbitMQ、Redis、NATS等消息代理
- **事件驱动处理**：基于事件的异步处理机制
- **Webhook回调**：处理完成后自动回调您的应用
- **重试机制**：自动重试失败的请求
- **死信队列**：处理无法成功处理的消息

## 快速开始

### 1. 基础配置

在您的Proksi配置文件中添加异步API插件：

```hcl
// 在路由的plugins部分
async_api {
  config_name = "default_async"
  
  config {
    enabled = true
    
    // Webhook配置
    webhook_config {
      enabled = true
      default_timeout_ms = 30000
      max_retries = 3
      retry_delay_ms = 1000
      signature_header = "X-Signature"
      secret_key = "your-webhook-secret"
    }
    
    // 重试配置
    retry_config {
      enabled = true
      max_attempts = 3
      initial_delay_ms = 1000
      max_delay_ms = 60000
      backoff_multiplier = 2.0
      retry_on_status_codes = [500, 502, 503, 504]
    }
    
    // 死信队列配置
    dead_letter_config {
      enabled = true
      topic_suffix = "_dlq"
      max_retries = 5
      retention_hours = 168  // 7天
    }
  }
}
```

### 2. 发送异步请求

要发送异步请求，只需在HTTP头中添加`X-Async-Processing: true`：

```bash
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "X-Async-Processing: true" \
  -H "X-Event-Type: ai_chat_request" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {
        "role": "user", 
        "content": "Write a detailed analysis of renewable energy trends"
      }
    ]
  }'
```

### 3. 处理响应

异步请求会立即返回一个接受响应：

```json
{
  "status": "accepted",
  "message_id": "msg_abc123def456",
  "request_id": "req_789xyz012"
}
```

## 消息代理集成

### Kafka集成

```hcl
message_brokers = [
  {
    name = "kafka_primary"
    broker_type = "kafka"
    connection_string = "localhost:9092"
    enabled = true
    
    topics = [
      {
        name = "ai_requests"
        partitions = 6
        replication_factor = 2
        retention_ms = 604800000  // 7天
        cleanup_policy = "delete"
      },
      {
        name = "ai_responses"
        partitions = 6
        replication_factor = 2
      }
    ]
    
    consumer_groups = [
      {
        group_id = "ai_gateway_processors"
        topics = ["ai_requests"]
        auto_offset_reset = "earliest"
        enable_auto_commit = true
        max_poll_records = 100
      }
    ]
    
    producer_config {
      acks = "all"
      retries = 3
      batch_size = 16384
      linger_ms = 5
      buffer_memory = 33554432
    }
  }
]
```

### RabbitMQ集成

```hcl
message_brokers = [
  {
    name = "rabbitmq_primary"
    broker_type = "rabbitmq"
    connection_string = "amqp://user:password@localhost:5672/"
    enabled = true
    
    topics = [
      {
        name = "ai.requests.queue"
        # RabbitMQ特定配置
      }
    ]
  }
]
```

### Redis集成

```hcl
message_brokers = [
  {
    name = "redis_primary"
    broker_type = "redis"
    connection_string = "redis://localhost:6379/0"
    enabled = true
    
    topics = [
      {
        name = "ai_requests_stream"
      }
    ]
  }
]
```

## 事件处理器

### Webhook处理器

Webhook处理器在消息处理完成后调用您的端点：

```hcl
event_handlers = [
  {
    name = "completion_webhook"
    event_type = "ai_request_completed"
    handler_type = "webhook"
    endpoint = "https://your-app.com/webhooks/ai-completed"
    headers = {
      "Authorization" = "Bearer your-token"
      "Content-Type" = "application/json"
    }
    timeout_ms = 10000
    enabled = true
  },
  {
    name = "error_webhook"
    event_type = "ai_request_failed"
    handler_type = "webhook"
    endpoint = "https://your-app.com/webhooks/ai-failed"
    timeout_ms = 5000
    enabled = true
  }
]
```

### 数据库处理器

将结果直接写入数据库：

```hcl
event_handlers = [
  {
    name = "db_logger"
    event_type = "ai_request_completed"
    handler_type = "database"
    endpoint = "postgresql://user:pass@localhost/aidb"
    timeout_ms = 5000
    enabled = true
  }
]
```

### 自定义处理器

```hcl
event_handlers = [
  {
    name = "metrics_collector"
    event_type = "ai_request_completed"
    handler_type = "custom"
    # 自定义处理器的配置
    enabled = true
  }
]
```

## API端点

### 查询消息状态

```bash
GET /async-api/messages/{message_id}/status
```

响应示例：
```json
{
  "message_id": "msg_abc123def456",
  "status": "completed",
  "processed_at": "2024-01-15T10:30:00Z",
  "retry_count": 0,
  "processing_time_ms": 2500,
  "result": {
    "choices": [
      {
        "message": {
          "role": "assistant",
          "content": "Renewable energy trends analysis..."
        }
      }
    ]
  }
}
```

### 获取队列统计

```bash
GET /async-api/stats
```

响应示例：
```json
{
  "pending_messages": 15,
  "processing_messages": 3,
  "completed_messages": 1247,
  "failed_messages": 12,
  "dead_letter_messages": 2,
  "total_messages": 1279
}
```

### 重新处理死信消息

```bash
POST /async-api/dead-letter/{message_id}/retry
```

### 清理过期消息

```bash
POST /async-api/cleanup
```

## 高级用例

### 批量处理

发送多个异步请求进行批量处理：

```bash
#!/bin/bash

# 批量发送请求
for i in {1..100}; do
  curl -X POST http://localhost:8000/v1/chat/completions \
    -H "Content-Type: application/json" \
    -H "X-Async-Processing: true" \
    -H "X-Event-Type: batch_ai_request" \
    -H "X-Correlation-Id: batch_001" \
    -d "{
      \"model\": \"gpt-3.5-turbo\",
      \"messages\": [
        {\"role\": \"user\", \"content\": \"Process item $i\"}
      ]
    }" &
done

wait
echo "All requests submitted"
```

### 工作流处理

使用相关ID链接相关的异步请求：

```bash
# 第一步：数据分析
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "X-Async-Processing: true" \
  -H "X-Event-Type: data_analysis" \
  -H "X-Correlation-Id: workflow_123" \
  -d '{"model": "gpt-4", "messages": [...]}'

# 第二步：报告生成（在第一步完成后触发）
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "X-Async-Processing: true" \
  -H "X-Event-Type: report_generation" \
  -H "X-Correlation-Id: workflow_123" \
  -H "X-Depends-On: data_analysis" \
  -d '{"model": "gpt-4", "messages": [...]}'
```

### 优先级处理

使用不同的事件类型实现优先级处理：

```bash
# 高优先级请求
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "X-Async-Processing: true" \
  -H "X-Event-Type: high_priority_request" \
  -H "X-Priority: high" \
  -d '{"model": "gpt-4", "messages": [...]}'

# 低优先级请求
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "X-Async-Processing: true" \
  -H "X-Event-Type: low_priority_request" \
  -H "X-Priority: low" \
  -d '{"model": "gpt-3.5-turbo", "messages": [...]}'
```

## 监控和调试

### 日志分析

```bash
# 查看异步处理日志
tail -f /var/log/proksi/async_api.log

# 过滤特定事件类型
grep "ai_request_completed" /var/log/proksi/async_api.log

# 查看错误日志
grep "ERROR" /var/log/proksi/async_api.log | tail -20
```

### 性能监控

```bash
# 监控队列长度
watch -n 5 'curl -s http://localhost:8000/async-api/stats | jq .pending_messages'

# 监控处理时间
curl -s http://localhost:8000/async-api/stats | jq .avg_processing_time_ms

# 监控错误率
curl -s http://localhost:8000/async-api/stats | jq '.failed_messages / .total_messages'
```

### 健康检查

```bash
# 检查异步API健康状态
curl http://localhost:8000/async-api/health

# 检查消息代理连接
curl http://localhost:8000/async-api/brokers/status
```

## 最佳实践

### 1. 消息设计

- 使用有意义的事件类型名称
- 包含足够的上下文信息
- 设置合适的相关ID用于追踪

### 2. 错误处理

- 配置合适的重试策略
- 监控死信队列
- 设置告警机制

### 3. 性能优化

- 根据负载调整队列大小
- 使用批处理提高吞吐量
- 监控和调整超时设置

### 4. 安全考虑

- 验证Webhook签名
- 使用HTTPS端点
- 限制访问权限

## 故障排除

### 常见问题

1. **消息处理缓慢**
   - 检查消息代理连接
   - 增加处理器数量
   - 优化事件处理器性能

2. **Webhook调用失败**
   - 验证端点可访问性
   - 检查签名验证
   - 查看超时设置

3. **消息丢失**
   - 检查消息代理配置
   - 验证持久化设置
   - 查看死信队列

### 调试技巧

```bash
# 启用详细日志
export RUST_LOG=proksi::plugins::async_api=debug

# 测试Webhook端点
curl -X POST https://your-app.com/webhooks/test \
  -H "Content-Type: application/json" \
  -d '{"test": "message"}'

# 检查消息代理连接
telnet localhost 9092  # Kafka
telnet localhost 5672  # RabbitMQ
```

## 扩展阅读

- [性能优化指南](./performance_optimization.md)
- [消息队列最佳实践](./message_queue_best_practices.md)
- [监控和告警配置](./monitoring_guide.md)
