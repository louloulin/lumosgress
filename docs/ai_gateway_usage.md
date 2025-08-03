# Proksi AI网关使用指南

Proksi AI网关是一个功能强大的反向代理解决方案，专为大型语言模型(LLM)流量管理而设计。通过Proksi AI网关，您可以高效地路由、转换、安全检查并聚合与LLM提供商之间的请求。

## 核心功能

- **多LLM提供商支持**：支持OpenAI、Anthropic、Google Vertex AI和Azure OpenAI等主流LLM提供商
- **智能路由**：基于请求内容和语义智能路由到最合适的LLM
- **提示转换**：自动增强和转换提示，添加系统消息、上下文和安全指令
- **AI安全检查**：检测提示注入、敏感信息和执行速率限制
- **模型聚合**：结合多个LLM提供商的结果或选择最快响应

## 安装

Proksi可以通过多种方式安装，包括直接二进制下载、Docker或从源代码构建。

### 二进制安装

```bash
curl -fsSL https://proksi.example.com/install.sh | sh
```

### Docker安装

```bash
docker pull proksi/proksi:latest
docker run -p 8000:8000 -v $(pwd)/config.hcl:/etc/proksi/config.hcl proksi/proksi
```

### 从源代码构建

```bash
git clone https://github.com/luizfonseca/proksi.git
cd proksi
cargo build --release
```

## 配置指南

Proksi使用HCL格式进行配置。以下是配置AI网关的关键部分。

### 1. LLM路由器

LLM路由器允许您将请求路由到不同的LLM提供商。

```hcl
// 在路由的plugins部分
llm_router {
  config_name = "default_router"
  
  config {
    default_provider = "openai"
    semantic_routing_enabled = true
    
    providers {
      "openai" = {
        endpoint = "api.openai.com/v1/chat/completions"
        models = ["gpt-3.5-turbo", "gpt-4"]
      }
      "anthropic" = {
        endpoint = "api.anthropic.com/v1/messages"
        models = ["claude-2", "claude-instant-1"]
      }
    }
  }
}
```

**核心参数**:
- `default_provider`: 默认使用的LLM提供商
- `semantic_routing_enabled`: 启用基于内容的智能路由
- `providers`: 配置可用的LLM提供商及其端点

### 2. 提示转换器

提示转换器可以自动增强您的提示，添加系统消息、上下文信息或安全指令。

```hcl
prompt_transform {
  config_name = "enhance_prompts"
  
  config {
    transformations = [
      {
        transform_type = "system_message"
        content = "You are a helpful AI assistant that provides accurate information."
      },
      {
        transform_type = "safety_check"
        content = "Ensure your response is ethical and does not contain harmful content."
      }
    ]
  }
}
```

**转换类型**:
- `system_message`: 添加系统指令
- `context`: 添加上下文信息
- `safety_check`: 添加安全指令
- `template`: 应用预定义模板

### 3. AI安全

AI安全插件提供多层保护，防止提示注入、敏感信息泄露和滥用。

```hcl
ai_security {
  config_name = "default_security"
  
  config {
    policies = [
      {
        policy_type = "prompt_injection"
        action = "block"
        patterns = [
          "ignore previous instructions",
          "disregard safety"
        ]
      },
      {
        policy_type = "token_limit"
        action = "block"
        max_tokens = 4000
      },
      {
        policy_type = "rate_limit"
        action = "block"
        max_requests = 60
        time_window = 60
      }
    ]
  }
}
```

**安全策略类型**:
- `prompt_injection`: 检测提示注入尝试
- `jailbreak_detection`: 检测越狱攻击（新增）
- `malicious_code_detection`: 检测恶意代码（新增）
- `data_leakage_prevention`: 防止数据泄露（新增）
- `output_filtering`: 输出内容过滤（新增）
- `sensitive_info`: 检测敏感信息（如信用卡、社保号）
- `token_limit`: 限制请求中的token数量
- `rate_limit`: 限制请求频率

**可用操作**:
- `block`: 阻止请求
- `log`: 仅记录但不阻止
- `sanitize`: 尝试清理问题内容
- `quarantine`: 隔离请求进行进一步分析（新增）
- `redirect`: 重定向到安全端点（新增）

### 4. LLM聚合器

LLM聚合器允许您并行调用多个LLM提供商，并以不同方式组合结果。

```hcl
llm_aggregator {
  config_name = "multi_provider"
  
  config {
    strategy = "fastest_result"  // 或 "combine_results", "chain"
    providers = ["openai", "anthropic"]
    timeout_ms = 10000
    weights = {
      "openai" = 0.7
      "anthropic" = 0.3
    }
  }
}
```

**聚合策略**:
- `fastest_result`: 返回最快的结果
- `combine_results`: 合并多个提供商的结果
- `chain`: 串联调用多个提供商，后一个处理前一个的输出

## 完整配置示例

请参考项目根目录下的[examples/ai_gateway_config.hcl](../examples/ai_gateway_config.hcl)文件，查看完整的配置示例。

## API使用示例

使用Proksi AI网关与直接调用LLM API类似，只是您将请求发送到Proksi端点而不是直接发送到LLM提供商。

```bash
# 通过Proksi使用OpenAI
curl -X POST https://api.openai.proxy/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $YOUR_API_KEY" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {"role": "user", "content": "Tell me about AI gateways"}
    ]
  }'
```

## 高级用例

### 混合模型响应

使用聚合器端点获取多个模型的组合结果：

```bash
curl -X POST https://llm.aggregator.ai/aggregate/combine \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $YOUR_API_KEY" \
  -d '{
    "prompt": "Explain quantum computing",
    "providers": ["openai", "anthropic"]
  }'
```

### 自定义提示转换

为特定请求自定义提示转换：

```bash
curl -X POST https://api.openai.proxy/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $YOUR_API_KEY" \
  -H "X-Transform-Template: expert_mode" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {"role": "user", "content": "Explain CRISPR technology"}
    ]
  }'
```

## 故障排除

### 常见问题

1. **请求被AI安全插件阻止**
   - 检查请求中是否包含可能被视为提示注入的内容
   - 查看日志了解具体阻止原因

2. **LLM路由不正确**
   - 确认配置中的默认提供商设置
   - 检查semantic_routing_enabled是否启用

3. **提示转换不生效**
   - 验证请求Content-Type是否为application/json
   - 检查提示格式是否与预期的格式匹配

### 日志分析

Proksi提供详细的日志，可帮助诊断问题：

```bash
# 查看最近的错误日志
tail -f /var/log/proksi/error.log | grep "ai_gateway"

# 查看特定插件的日志
tail -f /var/log/proksi/access.log | grep "llm_router"
```

## 性能优化

- 调整worker_threads配置以匹配服务器CPU核心数
- 对频繁请求的路由启用缓存
- 对compute-heavy请求使用智能负载均衡
- 考虑将Proksi部署在与LLM API相同的地区以减少延迟

## 扩展阅读

- [Proksi架构概述](./architecture.md)
- [AI网关最佳实践](./ai_gateway_best_practices.md)
- [安全配置指南](./security_guide.md)
- [API参考文档](./api_reference.md) 