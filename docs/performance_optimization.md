# Proksi AI网关性能优化指南

本指南介绍如何优化Proksi AI网关的性能，包括缓存策略、请求优化、响应时间监控等高级功能。

## 性能分析器插件

性能分析器插件提供了全面的性能监控和优化建议功能。

### 基础配置

```hcl
performance_analyzer {
  config_name = "default_performance"
  
  config {
    enabled = true
    ui_endpoint = "/performance-analyzer"
    sample_rate = 1.0  // 采样率：1.0表示监控所有请求
    detailed_profiling = true
    hotspot_detection = true
    profile_token_usage = true
    
    // 新增：自动优化配置
    auto_optimization = true
    
    // 优化阈值配置
    optimization_thresholds {
      response_time_ms = 2000.0      // 响应时间阈值
      error_rate_percent = 5.0       // 错误率阈值
      token_efficiency = 0.8         // Token效率阈值
      cost_per_request = 0.01        // 每请求成本阈值
      throughput_rps = 100.0         // 吞吐量阈值
    }
    
    // 缓存配置
    caching_config {
      enabled = true
      ttl_seconds = 3600             // 1小时TTL
      max_cache_size = 10000         // 最大缓存10000条
      cache_key_strategy = "semantic_hash"  // 语义哈希策略
      cache_hit_threshold = 0.7      // 70%缓存命中率阈值
    }
    
    // 批处理配置
    batching_config {
      enabled = false                // 默认关闭批处理
      max_batch_size = 10
      batch_timeout_ms = 100         // 100ms批处理超时
      compatible_models = ["gpt-3.5-turbo", "gpt-4"]
    }
    
    // 模型选择配置
    model_selection_config {
      enabled = true
      auto_fallback = true
      
      // 性能权重配置
      performance_weights {
        speed = 0.3        // 30%权重给速度
        accuracy = 0.4     // 40%权重给准确性
        cost = 0.2         // 20%权重给成本
        reliability = 0.1  // 10%权重给可靠性
      }
    }
  }
}
```

### 缓存策略

性能分析器支持多种缓存策略：

#### 1. 完整请求缓存
```hcl
caching_config {
  cache_key_strategy = "full_request"
  ttl_seconds = 1800  // 30分钟
}
```

#### 2. 仅提示词缓存
```hcl
caching_config {
  cache_key_strategy = "prompt_only"
  ttl_seconds = 3600  // 1小时
}
```

#### 3. 语义哈希缓存（推荐）
```hcl
caching_config {
  cache_key_strategy = "semantic_hash"
  ttl_seconds = 7200  // 2小时
}
```

语义哈希缓存可以识别语义相似的请求，即使措辞略有不同也能命中缓存。

### 性能监控API

性能分析器提供了丰富的API端点用于监控和分析：

#### 获取性能指标
```bash
curl http://localhost:8000/performance-analyzer/api/metrics
```

#### 获取热点分析
```bash
curl http://localhost:8000/performance-analyzer/api/hotspots
```

#### 获取优化建议
```bash
curl http://localhost:8000/performance-analyzer/api/generate-optimizations
```

#### 获取模型性能排名
```bash
curl http://localhost:8000/performance-analyzer/api/model-performance
```

#### 获取缓存统计
```bash
curl http://localhost:8000/performance-analyzer/api/cache-stats
```

### 自动优化建议

性能分析器会自动分析系统性能并提供优化建议：

#### 响应时间优化
当平均响应时间超过阈值时，系统会建议：
- 启用缓存
- 优化模型选择
- 调整负载均衡策略

#### Token效率优化
当Token效率低于阈值时，系统会建议：
- 优化提示词长度
- 使用更精确的指令
- 减少不必要的上下文

#### 错误率优化
当错误率过高时，系统会建议：
- 实施智能负载均衡
- 添加重试机制
- 改进错误处理

#### 模型选择优化
系统会分析各模型的性能表现，建议使用：
- 响应时间最快的模型
- 成功率最高的模型
- 成本效益最优的模型

## 异步API支持

新增的异步API支持可以显著提高系统吞吐量和响应性能。

### 基础配置

```hcl
async_api {
  config_name = "default_async"
  
  config {
    enabled = true
    
    // 消息代理配置
    message_brokers = [
      {
        name = "kafka_broker"
        broker_type = "kafka"
        connection_string = "localhost:9092"
        enabled = true
        
        topics = [
          {
            name = "ai_requests"
            partitions = 3
            replication_factor = 1
          }
        ]
        
        consumer_groups = [
          {
            group_id = "ai_gateway_consumers"
            topics = ["ai_requests"]
            auto_offset_reset = "earliest"
          }
        ]
      }
    ]
    
    // 事件处理器配置
    event_handlers = [
      {
        name = "webhook_handler"
        event_type = "ai_request_completed"
        handler_type = "webhook"
        endpoint = "https://your-app.com/webhooks/ai-completed"
        timeout_ms = 5000
        enabled = true
      }
    ]
    
    // Webhook配置
    webhook_config {
      enabled = true
      default_timeout_ms = 30000
      max_retries = 3
      retry_delay_ms = 1000
    }
    
    // 重试配置
    retry_config {
      enabled = true
      max_attempts = 3
      initial_delay_ms = 1000
      max_delay_ms = 60000
      backoff_multiplier = 2.0
    }
  }
}
```

### 异步请求处理

要使用异步处理，在请求头中添加：

```bash
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "X-Async-Processing: true" \
  -H "X-Event-Type: ai_chat_request" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {"role": "user", "content": "Explain quantum computing"}
    ]
  }'
```

响应将立即返回一个消息ID：

```json
{
  "status": "accepted",
  "message_id": "msg_12345",
  "request_id": "req_67890"
}
```

### 消息状态查询

```bash
curl http://localhost:8000/async-api/status/msg_12345
```

### 队列统计

```bash
curl http://localhost:8000/async-api/stats
```

## 性能调优最佳实践

### 1. 缓存策略选择

- **高重复性查询**：使用`semantic_hash`策略
- **精确匹配需求**：使用`full_request`策略
- **提示词变化较小**：使用`prompt_only`策略

### 2. 批处理优化

对于支持批处理的模型，启用批处理可以显著提高吞吐量：

```hcl
batching_config {
  enabled = true
  max_batch_size = 20
  batch_timeout_ms = 50
  compatible_models = ["gpt-3.5-turbo", "claude-instant"]
}
```

### 3. 模型选择策略

根据业务需求调整模型性能权重：

```hcl
// 注重速度的场景
performance_weights {
  speed = 0.6
  accuracy = 0.2
  cost = 0.1
  reliability = 0.1
}

// 注重准确性的场景
performance_weights {
  speed = 0.1
  accuracy = 0.7
  cost = 0.1
  reliability = 0.1
}

// 注重成本的场景
performance_weights {
  speed = 0.2
  accuracy = 0.3
  cost = 0.4
  reliability = 0.1
}
```

### 4. 监控和告警

设置性能监控告警：

```bash
# 监控响应时间
curl http://localhost:8000/performance-analyzer/api/metrics | \
  jq '.avg_response_time_ms > 2000'

# 监控错误率
curl http://localhost:8000/performance-analyzer/api/metrics | \
  jq '.error_rate > 0.05'

# 监控缓存命中率
curl http://localhost:8000/performance-analyzer/api/cache-stats | \
  jq '.cache_hit_rate < 0.7'
```

## 故障排除

### 常见性能问题

1. **响应时间过长**
   - 检查缓存配置和命中率
   - 分析模型选择策略
   - 考虑启用批处理

2. **内存使用过高**
   - 调整缓存大小限制
   - 检查是否有内存泄漏
   - 优化批处理配置

3. **CPU使用率过高**
   - 调整worker线程数
   - 检查是否有热点代码
   - 考虑使用异步处理

### 性能调试

启用详细的性能分析：

```hcl
performance_analyzer {
  config {
    detailed_profiling = true
    max_trace_depth = 20
    trace_headers = true
  }
}
```

查看详细的性能报告：

```bash
curl http://localhost:8000/performance-analyzer/ui
```

## 扩展阅读

- [AI网关架构设计](./architecture.md)
- [安全配置指南](./security_guide.md)
- [部署最佳实践](./deployment_guide.md)
