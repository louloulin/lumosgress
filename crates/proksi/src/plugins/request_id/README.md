# RequestId Plugin

## 功能概述
RequestId插件为每个请求生成并添加唯一的请求ID，并将其添加到响应头中。这有助于追踪请求，尤其是在微服务架构中，使得调试和日志分析更容易。

## 新接口实现
该插件已经从旧的`MiddlewarePlugin`接口迁移到新的统一`Plugin`接口。在新的实现中：

1. 添加了可配置的`RequestIdConfig`，支持：
   - 启用/禁用功能
   - 自定义请求ID的头部名称

2. 在`RouterContext`中利用新的`request_id`字段存储请求ID
   - 同时保持对`extensions`的向后兼容支持

3. 使用标准的Plugin生命周期方法：
   - `handle_request`: 在请求阶段生成并存储请求ID
   - `handle_response`: 在响应阶段添加请求ID头

## 使用方法

```rust
// 创建默认插件实例
let plugin = RequestId::new();

// 或者使用自定义配置
let plugin = RequestId::with_config(RequestIdConfig {
    enabled: true,
    header_name: "X-Custom-Request-ID".to_string(),
});
```

## 配置选项

```rust
pub struct RequestIdConfig {
    /// 是否启用请求ID功能
    pub enabled: bool,
    
    /// 请求ID使用的HTTP头名称
    pub header_name: String,
}
```

## 测试
已添加全面的单元测试，涵盖：
- 请求ID生成与存储
- 在响应头中设置请求ID
- 插件禁用时的行为

## 下一步改进
- 支持更多的ID生成算法（如nanoid）
- 添加插件的记录和指标统计功能
- 优化性能 