#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use pingora::proxy::Session;
    use crate::config::RoutePlugin;
    use crate::proxy_server::https_proxy::RouterContext;
    use crate::plugins::tenant::TenantPlugin;
    use crate::plugins::MiddlewarePlugin;
    use std::borrow::Cow;
    use serde_json::json;

    #[tokio::test]
    async fn test_tenant_plugin_config() {
        let tenant_plugin = TenantPlugin::new();
        let mut session = Session::new();
        let mut ctx = RouterContext {
            host: "test.example.com".to_string(),
            path: "/api".to_string(),
            extensions: HashMap::new(),
            metadata: HashMap::new(),
            route_container: crate::proxy_server::https_proxy::RouteContainer {
                plugins: HashMap::new(),
            },
        };
        
        // Create a test plugin configuration
        let mut config_map = HashMap::new();
        config_map.insert(Cow::Borrowed("isolation_enabled"), json!(true));
        config_map.insert(Cow::Borrowed("tenant_id_header"), json!("x-custom-tenant-id"));
        config_map.insert(Cow::Borrowed("default_quota"), json!({
            "requests": 50000,
            "tokens": 2000000
        }));
        
        let plugin = RoutePlugin {
            name: Cow::Borrowed("tenant"),
            config: Some(config_map),
        };
        
        // Test request filter
        let result = tenant_plugin.request_filter(&mut session, &mut ctx, &plugin).await;
        assert!(result.is_ok());
    }
} 