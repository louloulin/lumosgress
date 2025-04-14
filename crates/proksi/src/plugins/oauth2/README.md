# OAuth2 Plugin

## 功能概述
OAuth2插件提供了与第三方OAuth提供商集成的认证功能，允许用户通过安全的OAuth流程进行身份验证。目前支持GitHub和WorkOS作为OAuth提供商，未来可扩展支持更多提供商。

## 新接口实现
该插件已经从旧的`MiddlewarePlugin`接口迁移到新的统一`Plugin`接口。在新的实现中：

1. 添加了可配置的`OAuth2Config`，支持：
   - 指定OAuth提供商类型
   - 设置客户端ID和密钥
   - 自定义JWT密钥和重定向URL
   - 配置用户验证规则
   - 启用/禁用功能

2. 处理完整的OAuth2流程：
   - 检查当前用户是否已认证
   - 未认证时重定向到OAuth提供商
   - 处理OAuth回调并验证用户身份
   - 存储认证状态到安全cookie中

3. 使用标准的Plugin生命周期方法：
   - `handle_request`: 在EarlyRequest阶段处理OAuth认证
   - `handle_response`: 不需要响应处理

## 使用方法

```rust
// 创建默认插件实例
let plugin = Oauth2::new();

// 或者使用自定义配置
let plugin = Oauth2::with_config(OAuth2Config {
    provider: "github".to_string(),
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    jwt_secret: "your-jwt-secret".to_string(),
    redirect_url: Some("https://your-site.com/callback".to_string()),
    validations: Some(serde_json::json!({
        "organizations": ["your-org"]
    })),
    enabled: true,
});
```

## 配置选项

```rust
pub struct OAuth2Config {
    /// 提供商类型 (github, workos等)
    pub provider: String,
    
    /// 来自OAuth提供商的客户端ID
    pub client_id: String,
    
    /// 来自OAuth提供商的客户端密钥
    pub client_secret: String,
    
    /// 用于签名cookie的JWT密钥
    pub jwt_secret: String,
    
    /// 认证成功后的重定向URL
    pub redirect_url: Option<String>,
    
    /// 额外的验证规则
    pub validations: Option<serde_json::Value>,
    
    /// 是否启用此插件
    pub enabled: bool,
}
```

## 支持的提供商
- **GitHub**: 支持基于组织和团队的验证
- **WorkOS**: 支持企业SSO和目录同步

## 下一步改进
- 支持更多OAuth提供商（如Google、Microsoft等）
- 改进OAuth回调处理逻辑
- 添加更细粒度的权限控制
- 优化令牌管理和刷新逻辑 